use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Transfers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Transfers::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Transfers::UserId).big_integer().not_null())
                    .col(ColumnDef::new(Transfers::FromBankId).string().not_null())
                    .col(ColumnDef::new(Transfers::ToBankId).string().not_null())
                    .col(ColumnDef::new(Transfers::Amount).double().not_null())
                    .col(ColumnDef::new(Transfers::Currency).string().not_null())
                    .col(ColumnDef::new(Transfers::TransferDate).string().not_null())
                    .col(ColumnDef::new(Transfers::Description).string())
                    .col(ColumnDef::new(Transfers::SyncStatus).string().not_null())
                    .col(ColumnDef::new(Transfers::LastSyncedAt).string())
                    .col(ColumnDef::new(Transfers::CreatedAt).string())
                    .col(ColumnDef::new(Transfers::UpdatedAt).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Transfers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Transfers {
    Table,
    Id,
    UserId,
    FromBankId,
    ToBankId,
    Amount,
    Currency,
    TransferDate,
    Description,
    SyncStatus,
    LastSyncedAt,
    CreatedAt,
    UpdatedAt,
}
