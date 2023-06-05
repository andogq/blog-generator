use sea_orm_migration::prelude::*;

const FK_USER_SOURCE_USER_ID: &str = "fk_user_source_user_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create user table
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(User::Id)
                            .integer()
                            .not_null()
                            .uuid()
                            .extra("DEFAULT gen_random_uuid()".to_string())
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(User::Created)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(User::LastLogin)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create user source table
        manager
            .create_table(
                Table::create()
                    .table(UserSource::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UserSource::Username).string().not_null())
                    .col(ColumnDef::new(UserSource::Site).string().not_null())
                    .col(
                        ColumnDef::new(UserSource::Created)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(UserSource::UserId).uuid().not_null())
                    .col(ColumnDef::new(UserSource::Token).string().not_null())
                    .primary_key(
                        Index::create()
                            .col(UserSource::Username)
                            .col(UserSource::Site),
                    )
                    .to_owned(),
            )
            .await?;

        // Create foreign key between user source and user
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_USER_SOURCE_USER_ID)
                    .from(UserSource::Table, UserSource::UserId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name(FK_USER_SOURCE_USER_ID)
                    .table(UserSource::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserSource::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum User {
    Table,
    Id,
    Created,
    LastLogin,
}

#[derive(Iden)]
enum UserSource {
    Table,
    UserId,
    Created,
    Site,
    Username,
    Token,
}
