//! Per-account statement: the full trail of rows that moved an account, newest first,
//! with a running balance walked backwards from the stored balance.
//!
//! `current_balance` stays the single source of truth — this module never recomputes it.
//! It starts at the stored value and subtracts each entry's signed amount as it walks
//! older, so the newest `balance_after` always reads as the balance the user sees today
//! and any manual balance edit simply becomes the anchor the trail hangs from.
//!
//! Sign convention mirrors `src/ledger.rs`: an income credits a bank, an expense debits a
//! bank and charges a card, an asset or investment purchase debits the bank it was paid
//! from. Two consequences follow, and both are deliberate:
//!
//! * An expense that carries a card id is charged to the card even when it also names a
//!   bank (the card wins), so it appears on the card's statement and not the bank's —
//!   listing it on the bank side would put a movement in the trail that never happened.
//! * A buy-list conversion writes an expense *and* an asset for one purchase and debits
//!   the account exactly once, so the asset half of that pair is dropped and the expense
//!   is what the statement shows. Without this the same outlay would be counted twice.

use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use serde::Serialize;

use crate::models::{asset_buy_list, assets, expenses, incomes, investments};

/// Money is `f64` in the schema, so round at every boundary. `-0.0` normalises to `0.0`.
fn round2(v: f64) -> f64 {
    let r = (v * 100.0).round() / 100.0;
    if r == 0.0 {
        0.0
    } else {
        r
    }
}

/// Blank and `"0"` ids mean "not linked" — the same rule `ledger::linked` applies.
fn linked(id: Option<&str>) -> Option<&str> {
    let s = id?.trim();
    if s.is_empty() || s == "0" {
        None
    } else {
        Some(s)
    }
}

/// One line of the statement, as returned by the endpoints.
#[derive(Clone, Debug, Serialize)]
pub struct StatementEntry {
    pub entry_type: String,
    pub id: String,
    pub title: String,
    pub amount: f64,
    pub signed_amount: f64,
    pub occurred_on: String,
    pub balance_after: f64,
}

/// Intermediate row before the running balance is walked on.
struct Row {
    entry_type: &'static str,
    id: String,
    title: String,
    amount: f64,
    signed: f64,
    occurred_on: String,
    created_at: String,
}

impl Row {
    fn push(
        out: &mut Vec<Row>,
        entry_type: &'static str,
        id: String,
        title: String,
        amount: f64,
        signed: f64,
        occurred_on: String,
        created_at: Option<String>,
    ) {
        out.push(Row {
            entry_type,
            id,
            title,
            amount,
            signed,
            occurred_on,
            created_at: created_at.unwrap_or_default(),
        });
    }
}

/// Sort newest first: date, then creation stamp, then id so ties are deterministic.
fn sort_newest_first(rows: &mut [Row]) {
    rows.sort_by(|a, b| {
        b.occurred_on
            .cmp(&a.occurred_on)
            .then_with(|| b.created_at.cmp(&a.created_at))
            .then_with(|| b.id.cmp(&a.id))
    });
}

/// Walk the running balance backwards from the account's current stored balance.
fn walk(current_balance: f64, mut rows: Vec<Row>) -> Vec<StatementEntry> {
    sort_newest_first(&mut rows);
    let mut balance = round2(current_balance);
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let signed = round2(r.signed);
        out.push(StatementEntry {
            entry_type: r.entry_type.to_string(),
            id: r.id,
            title: r.title,
            amount: round2(r.amount),
            signed_amount: signed,
            occurred_on: r.occurred_on,
            balance_after: balance,
        });
        balance = round2(balance - signed);
    }
    out
}

/// Assets created by a buy-list conversion: their expense already carries the outlay.
async fn converted_asset_ids(
    db: &impl ConnectionTrait,
    user_id: i64,
) -> Result<std::collections::HashSet<String>, sea_orm::DbErr> {
    Ok(asset_buy_list::Entity::find()
        .filter(asset_buy_list::Column::UserId.eq(user_id))
        .filter(asset_buy_list::Column::ConvertedAssetId.is_not_null())
        .all(db)
        .await?
        .into_iter()
        .filter_map(|r| r.converted_asset_id)
        .collect())
}

/// Full trail for a bank account, newest first.
pub async fn for_bank(
    db: &impl ConnectionTrait,
    user_id: i64,
    bank_id: &str,
    current_balance: f64,
) -> Result<Vec<StatementEntry>, sea_orm::DbErr> {
    let mut rows: Vec<Row> = Vec::new();

    for i in incomes::Entity::find()
        .filter(incomes::Column::UserId.eq(user_id))
        .filter(incomes::Column::BankId.eq(bank_id))
        .all(db)
        .await?
    {
        Row::push(
            &mut rows,
            "income",
            i.id,
            i.title,
            i.amount,
            i.amount,
            i.income_date,
            i.created_at,
        );
    }

    for e in expenses::Entity::find()
        .filter(expenses::Column::UserId.eq(user_id))
        .filter(expenses::Column::BankId.eq(bank_id))
        .all(db)
        .await?
    {
        // The card wins when both ids are present, so this expense never touched the bank.
        if linked(e.credit_card_id.as_deref()).is_some() {
            continue;
        }
        Row::push(
            &mut rows,
            "expense",
            e.id,
            e.title,
            e.amount,
            -e.amount,
            e.expense_date,
            e.created_at,
        );
    }

    let converted = converted_asset_ids(db, user_id).await?;
    for a in assets::Entity::find()
        .filter(assets::Column::UserId.eq(user_id))
        .filter(assets::Column::BankId.eq(bank_id))
        .all(db)
        .await?
    {
        if converted.contains(&a.id) {
            continue;
        }
        Row::push(
            &mut rows,
            "asset",
            a.id,
            a.name,
            a.purchase_price,
            -a.purchase_price,
            a.purchase_date,
            a.created_at,
        );
    }

    for inv in investments::Entity::find()
        .filter(investments::Column::UserId.eq(user_id))
        .filter(investments::Column::BankId.eq(bank_id))
        .all(db)
        .await?
    {
        Row::push(
            &mut rows,
            "investment",
            inv.id,
            inv.name,
            inv.invested_amount,
            -inv.invested_amount,
            inv.purchase_date,
            inv.created_at,
        );
    }

    Ok(walk(current_balance, rows))
}

/// Full trail for a credit card, newest first. Only expenses carry a card id, and a
/// charge raises the amount owed, so every signed amount here is positive.
pub async fn for_card(
    db: &impl ConnectionTrait,
    user_id: i64,
    card_id: &str,
    current_balance: f64,
) -> Result<Vec<StatementEntry>, sea_orm::DbErr> {
    let mut rows: Vec<Row> = Vec::new();

    for e in expenses::Entity::find()
        .filter(expenses::Column::UserId.eq(user_id))
        .filter(expenses::Column::CreditCardId.eq(card_id))
        .all(db)
        .await?
    {
        if linked(e.credit_card_id.as_deref()) != Some(card_id) {
            continue;
        }
        Row::push(
            &mut rows,
            "expense",
            e.id,
            e.title,
            e.amount,
            e.amount,
            e.expense_date,
            e.created_at,
        );
    }

    Ok(walk(current_balance, rows))
}
