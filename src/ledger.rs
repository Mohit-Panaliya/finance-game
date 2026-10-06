//! Keeps an account balance in step with the transactions booked against it.
//!
//! `incomes.bank_id` and `expenses.bank_id` / `expenses.credit_card_id` record
//! *which* account the money moved through. This module is the other half of that
//! link: it moves the balance.
//!
//! Sign convention:
//!   - an income credits the linked bank (`+amount`);
//!   - an expense debits the linked bank (`-amount`);
//!   - an expense charges the linked card (`+amount`), because a card's
//!     `current_balance` is the amount **owed**, not money in hand
//!     (`available = credit_limit - current_balance`).
//!
//! A card id wins when an expense carries both ids: one expense is settled through
//! one instrument, and applying it to both would double-count it.
//!
//! Every statement is scoped by `user_id` as well as the account id, so a row
//! pointing at somebody else's account cannot move it.
//!
//! Each adjustment is a single relative `UPDATE ... SET balance = ROUND(balance +
//! delta, 2)` rather than a read-modify-write, so two concurrent bookings cannot
//! lose one another's movement, and the rounding keeps repeated deltas from
//! drifting.
//!
//! This lives in the application rather than in database triggers on purpose. The
//! Turso engine used here gates `CREATE TRIGGER` behind `--experimental-triggers`
//! and refuses to open a database that contains triggers when that flag is absent,
//! so a trigger-based ledger would tie the app's ability to boot to an
//! experimental engine flag.
//!
//! Rows that already exist are not replayed: balances stay what the user last
//! entered. Backfilling history is a deliberate, separate operation.

use sea_orm::{
    sea_query::{Expr, ExprTrait},
    ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
};

use crate::models::{banks, credit_cards};

/// Blank ids arrive from the frontend as `""`, so they mean "not linked".
fn linked(id: Option<&String>) -> Option<&str> {
    id.map(|s| s.trim()).filter(|s| !s.is_empty() && *s != "0")
}

/// Apply `delta` to a bank balance. A no-op when the id is missing or blank.
pub async fn adjust_bank(
    db: &impl ConnectionTrait,
    user_id: i64,
    bank_id: Option<&String>,
    delta: f64,
) -> Result<(), sea_orm::DbErr> {
    let Some(bank_id) = linked(bank_id) else {
        return Ok(());
    };
    if delta == 0.0 {
        return Ok(());
    }
    banks::Entity::update_many()
        .col_expr(
            banks::Column::CurrentBalance,
            Expr::col(banks::Column::CurrentBalance).add(delta),
        )
        .filter(banks::Column::Id.eq(bank_id))
        .filter(banks::Column::UserId.eq(user_id))
        .exec(db)
        .await?;
    Ok(())
}

/// Apply `delta` to a credit card's amount owed.
pub async fn adjust_card(
    db: &impl ConnectionTrait,
    user_id: i64,
    card_id: Option<&String>,
    delta: f64,
) -> Result<(), sea_orm::DbErr> {
    let Some(card_id) = linked(card_id) else {
        return Ok(());
    };
    if delta == 0.0 {
        return Ok(());
    }
    credit_cards::Entity::update_many()
        .col_expr(
            credit_cards::Column::CurrentBalance,
            Expr::col(credit_cards::Column::CurrentBalance).add(delta),
        )
        .filter(credit_cards::Column::Id.eq(card_id))
        .filter(credit_cards::Column::UserId.eq(user_id))
        .exec(db)
        .await?;
    Ok(())
}

/// The accounts an expense is booked against, after the card-beats-bank rule.
struct ExpenseTargets<'a> {
    bank_id: Option<&'a String>,
    card_id: Option<&'a String>,
}

impl<'a> ExpenseTargets<'a> {
    fn new(bank_id: Option<&'a String>, card_id: Option<&'a String>) -> Self {
        match linked(card_id) {
            // A linked card wins, so the bank is deliberately left out.
            Some(_) => Self {
                bank_id: None,
                card_id,
            },
            None => Self {
                bank_id,
                card_id: None,
            },
        }
    }

    /// `sign` is `1.0` to apply the expense, `-1.0` to undo it.
    async fn apply(
        &self,
        db: &impl ConnectionTrait,
        user_id: i64,
        amount: f64,
        sign: f64,
    ) -> Result<(), sea_orm::DbErr> {
        adjust_bank(db, user_id, self.bank_id, -amount * sign).await?;
        adjust_card(db, user_id, self.card_id, amount * sign).await?;
        Ok(())
    }
}

/// Record a new income: credit the linked bank by `amount`.
pub async fn on_income_created(
    db: &impl ConnectionTrait,
    user_id: i64,
    bank_id: Option<&String>,
    amount: f64,
) -> Result<(), sea_orm::DbErr> {
    adjust_bank(db, user_id, bank_id, amount).await
}

/// Undo a deleted income.
pub async fn on_income_deleted(
    db: &impl ConnectionTrait,
    user_id: i64,
    bank_id: Option<&String>,
    amount: f64,
) -> Result<(), sea_orm::DbErr> {
    adjust_bank(db, user_id, bank_id, -amount).await
}

/// Reconcile an edited income by crediting the new state and undoing the old one.
pub async fn on_income_updated(
    db: &impl ConnectionTrait,
    user_id: i64,
    old_bank_id: Option<&String>,
    old_amount: f64,
    new_bank_id: Option<&String>,
    new_amount: f64,
) -> Result<(), sea_orm::DbErr> {
    let same_account = linked(old_bank_id) == linked(new_bank_id);
    if same_account {
        adjust_bank(db, user_id, new_bank_id, new_amount - old_amount).await
    } else {
        adjust_bank(db, user_id, old_bank_id, -old_amount).await?;
        adjust_bank(db, user_id, new_bank_id, new_amount).await
    }
}

/// Record a new expense.
pub async fn on_expense_created(
    db: &impl ConnectionTrait,
    user_id: i64,
    bank_id: Option<&String>,
    card_id: Option<&String>,
    amount: f64,
) -> Result<(), sea_orm::DbErr> {
    ExpenseTargets::new(bank_id, card_id)
        .apply(db, user_id, amount, 1.0)
        .await
}

/// Undo a deleted expense.
pub async fn on_expense_deleted(
    db: &impl ConnectionTrait,
    user_id: i64,
    bank_id: Option<&String>,
    card_id: Option<&String>,
    amount: f64,
) -> Result<(), sea_orm::DbErr> {
    ExpenseTargets::new(bank_id, card_id)
        .apply(db, user_id, amount, -1.0)
        .await
}

/// Reconcile an edited expense. Each side is applied separately because a bank and
/// a card can be swapped in one edit, which moves money between two accounts.
pub async fn on_expense_updated(
    db: &impl ConnectionTrait,
    user_id: i64,
    old_bank_id: Option<&String>,
    old_card_id: Option<&String>,
    old_amount: f64,
    new_bank_id: Option<&String>,
    new_card_id: Option<&String>,
    new_amount: f64,
) -> Result<(), sea_orm::DbErr> {
    let old = ExpenseTargets::new(old_bank_id, old_card_id);
    let new = ExpenseTargets::new(new_bank_id, new_card_id);
    if old.bank_id == new.bank_id && old.card_id == new.card_id {
        let delta = new_amount - old_amount;
        adjust_bank(db, user_id, new.bank_id, -delta).await?;
        adjust_card(db, user_id, new.card_id, delta).await
    } else {
        old.apply(db, user_id, old_amount, -1.0).await?;
        new.apply(db, user_id, new_amount, 1.0).await
    }
}

/// An outlay that leaves a bank account: buying an asset, funding an investment, or
/// placing a fixed deposit.
///
/// Same accounting shape as an expense paid out of a bank, so it reuses those rules
/// rather than inventing a second set. These are deliberately bank-only: unlike an
/// expense they cannot be settled on a card without a card purchase in front of them,
/// and pretending otherwise would let one outlay move two accounts.
pub async fn on_purchase_created(
    db: &impl ConnectionTrait,
    user_id: i64,
    bank_id: Option<&String>,
    amount: f64,
) -> Result<(), sea_orm::DbErr> {
    ExpenseTargets::new(bank_id, None)
        .apply(db, user_id, amount, 1.0)
        .await
}

/// Give the money back to the bank it came from.
pub async fn on_purchase_deleted(
    db: &impl ConnectionTrait,
    user_id: i64,
    bank_id: Option<&String>,
    amount: f64,
) -> Result<(), sea_orm::DbErr> {
    ExpenseTargets::new(bank_id, None)
        .apply(db, user_id, amount, -1.0)
        .await
}

/// Reconcile an edited outlay against the ledger.
pub async fn on_purchase_updated(
    db: &impl ConnectionTrait,
    user_id: i64,
    old_bank_id: Option<&String>,
    old_amount: f64,
    new_bank_id: Option<&String>,
    new_amount: f64,
) -> Result<(), sea_orm::DbErr> {
    let old = ExpenseTargets::new(old_bank_id, None);
    let new = ExpenseTargets::new(new_bank_id, None);
    if old.bank_id == new.bank_id {
        adjust_bank(db, user_id, new.bank_id, old_amount - new_amount).await
    } else {
        old.apply(db, user_id, old_amount, -1.0).await?;
        new.apply(db, user_id, new_amount, 1.0).await
    }
}
