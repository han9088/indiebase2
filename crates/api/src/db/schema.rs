//! Platform schema definitions (SeaQuery). Source of truth for development synchronize.

use sea_query::{
    ColumnDef, ConditionalStatement, Expr, ExprTrait, ForeignKey, ForeignKeyAction, Iden, Index,
    PostgresQueryBuilder, Table,
};

#[derive(Iden)]
pub enum Users {
    Table,
    Id,
    Email,
    PasswordHash,
    CreatedAt,
    DeletedAt,
}

#[derive(Iden)]
pub enum Projects {
    Table,
    Id,
    Name,
    CreatedAt,
    DeletedAt,
}

#[derive(Iden)]
pub enum ProjectMembers {
    Table,
    ProjectId,
    UserId,
    Role,
    DeletedAt,
}

#[derive(Iden)]
pub enum ApiKeys {
    Table,
    Id,
    ProjectId,
    KeyType,
    KeyHash,
    KeyPrefix,
    Status,
    CreatedAt,
    DeletedAt,
}

fn ulid_pk(iden: impl Iden + 'static) -> ColumnDef {
    ColumnDef::new(iden)
        .char_len(26)
        .not_null()
        .primary_key()
        .take()
}

fn ulid_fk(iden: impl Iden + 'static) -> ColumnDef {
    ColumnDef::new(iden).char_len(26).not_null().take()
}

fn timestamptz_now(iden: impl Iden + 'static) -> ColumnDef {
    ColumnDef::new(iden)
        .timestamp_with_time_zone()
        .not_null()
        .default(Expr::cust("now()"))
        .take()
}

fn timestamptz_null(iden: impl Iden + 'static) -> ColumnDef {
    ColumnDef::new(iden)
        .timestamp_with_time_zone()
        .null()
        .take()
}

/// Rendered DDL statements for platform tables + indexes (no roles).
pub fn platform_table_statements() -> Vec<String> {
    let users = Table::create()
        .table(Users::Table)
        .if_not_exists()
        .col(ulid_pk(Users::Id))
        .col(ColumnDef::new(Users::Email).text().not_null())
        .col(ColumnDef::new(Users::PasswordHash).text().not_null())
        .col(timestamptz_now(Users::CreatedAt))
        .col(timestamptz_null(Users::DeletedAt))
        .to_owned()
        .to_string(PostgresQueryBuilder);

    let projects = Table::create()
        .table(Projects::Table)
        .if_not_exists()
        .col(ulid_pk(Projects::Id))
        .col(ColumnDef::new(Projects::Name).text().not_null())
        .col(timestamptz_now(Projects::CreatedAt))
        .col(timestamptz_null(Projects::DeletedAt))
        .to_owned()
        .to_string(PostgresQueryBuilder);

    let project_members = Table::create()
        .table(ProjectMembers::Table)
        .if_not_exists()
        .col(ulid_fk(ProjectMembers::ProjectId))
        .col(ulid_fk(ProjectMembers::UserId))
        .col(
            ColumnDef::new(ProjectMembers::Role)
                .text()
                .not_null()
                .check(Expr::cust("role IN ('owner', 'admin', 'member')")),
        )
        .col(timestamptz_null(ProjectMembers::DeletedAt))
        .primary_key(
            Index::create()
                .col(ProjectMembers::ProjectId)
                .col(ProjectMembers::UserId),
        )
        .foreign_key(
            ForeignKey::create()
                .name("project_members_project_id_fkey")
                .from(ProjectMembers::Table, ProjectMembers::ProjectId)
                .to(Projects::Table, Projects::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .foreign_key(
            ForeignKey::create()
                .name("project_members_user_id_fkey")
                .from(ProjectMembers::Table, ProjectMembers::UserId)
                .to(Users::Table, Users::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .to_owned()
        .to_string(PostgresQueryBuilder);

    let api_keys = Table::create()
        .table(ApiKeys::Table)
        .if_not_exists()
        .col(ulid_pk(ApiKeys::Id))
        .col(ulid_fk(ApiKeys::ProjectId))
        .col(
            ColumnDef::new(ApiKeys::KeyType)
                .text()
                .not_null()
                .check(Expr::cust("key_type IN ('publishable', 'secret')")),
        )
        .col(ColumnDef::new(ApiKeys::KeyHash).text().not_null())
        .col(ColumnDef::new(ApiKeys::KeyPrefix).text().not_null())
        .col(
            ColumnDef::new(ApiKeys::Status)
                .text()
                .not_null()
                .default("active")
                .check(Expr::cust("status IN ('active', 'disabled')")),
        )
        .col(timestamptz_now(ApiKeys::CreatedAt))
        .col(timestamptz_null(ApiKeys::DeletedAt))
        .foreign_key(
            ForeignKey::create()
                .name("api_keys_project_id_fkey")
                .from(ApiKeys::Table, ApiKeys::ProjectId)
                .to(Projects::Table, Projects::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .to_owned()
        .to_string(PostgresQueryBuilder);

    let users_email_uidx = Index::create()
        .if_not_exists()
        .unique()
        .name("users_email_active_uidx")
        .table(Users::Table)
        .col(Users::Email)
        .and_where(Expr::col(Users::DeletedAt).is_null())
        .to_owned()
        .to_string(PostgresQueryBuilder);

    let api_keys_project_idx = Index::create()
        .if_not_exists()
        .name("api_keys_project_id_idx")
        .table(ApiKeys::Table)
        .col(ApiKeys::ProjectId)
        .to_owned()
        .to_string(PostgresQueryBuilder);

    let api_keys_hash_idx = Index::create()
        .if_not_exists()
        .name("api_keys_key_hash_idx")
        .table(ApiKeys::Table)
        .col(ApiKeys::KeyHash)
        .to_owned()
        .to_string(PostgresQueryBuilder);

    vec![
        users,
        projects,
        project_members,
        api_keys,
        users_email_uidx,
        api_keys_project_idx,
        api_keys_hash_idx,
    ]
}

pub const TENANT_ROLES_SQL: &str = r#"
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'anon') THEN
        CREATE ROLE anon NOLOGIN;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'authenticated') THEN
        CREATE ROLE authenticated NOLOGIN;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'service') THEN
        CREATE ROLE service NOLOGIN BYPASSRLS;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'project_operator') THEN
        CREATE ROLE project_operator NOLOGIN BYPASSRLS;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'project_operator_readonly') THEN
        CREATE ROLE project_operator_readonly NOLOGIN;
    END IF;
END
$$;
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_ddl_includes_deleted_at_and_partial_email_index() {
        let sql = platform_table_statements().join("\n");
        assert!(sql.contains("deleted_at"));
        assert!(sql.contains("users_email_active_uidx"));
        assert!(sql.to_lowercase().contains("where"));
    }
}
