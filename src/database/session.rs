use std::time::{SystemTime, UNIX_EPOCH};

use crate::entity::{prelude::Session, session};
use sea_orm::*;

pub async fn create(
    rjti: String,
    user_id: String,
    expires: usize,
    db: &DatabaseConnection,
) -> Result<session::Model, DbErr> {
    Session::insert(session::ActiveModel {
        id: Set(rjti),
        user_id: Set(user_id),
        expires: Set(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + expires.try_into().unwrap_or(0)),
        ..Default::default()
    })
    .exec_with_returning(db)
    .await
}

pub async fn delete(rjti: String, db: &DatabaseConnection) -> Result<(), DbErr> {
    session::ActiveModel {
        id: Set(rjti),
        ..Default::default()
    }
    .delete(db)
    .await?;

    Ok(())
}

pub async fn delete_expired(user_id: &str, db: &DatabaseConnection) -> Result<(), DbErr> {
    let now: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| {
            DbErr::Custom(format!(
                "Failed to get the current time since epoch: {}",
                err
            ))
        })?
        .as_secs()
        .try_into()
        .map_err(|_| DbErr::Custom("Failed to convert the current time to seconds".to_string()))?;

    session::Entity::delete_many()
        .filter(session::Column::UserId.eq(user_id))
        .filter(session::Column::Expires.lte(now + 30))
        .exec(db)
        .await?;

    Ok(())
}
