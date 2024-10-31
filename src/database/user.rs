use crate::{
    entity::{prelude::User, user},
    model::user::UpdateUser,
};
use sea_orm::*;

pub async fn create(
    email: String,
    name: String,
    password: String,
    db: &DatabaseConnection,
) -> Result<user::Model, DbErr> {
    User::insert(user::ActiveModel {
        email: Set(email),
        name: Set(name),
        password: Set(password),
        ..Default::default()
    })
    .exec_with_returning(db)
    .await
}

pub async fn find_by_id(id: String, db: &DatabaseConnection) -> Result<Option<user::Model>, DbErr> {
    User::find_by_id(id).one(db).await
}

pub async fn find_by_email(
    email: String,
    db: &DatabaseConnection,
) -> Result<Option<user::Model>, DbErr> {
    User::find()
        .filter(user::Column::Email.eq(email))
        .one(db)
        .await
}

pub async fn update(data: UpdateUser, db: &DatabaseConnection) -> Result<user::Model, DbErr> {
    let mut update = user::ActiveModel {
        id: Set(data.id),
        ..Default::default()
    };

    if let Some(email) = data.email {
        update.email = Set(email);
    }
    if let Some(name) = data.name {
        update.name = Set(name);
    }
    if let Some(password) = data.password {
        update.password = Set(password);
    }

    update.save(db).await?.try_into_model()
}

pub async fn delete(id: String, db: &DatabaseConnection) -> Result<(), DbErr> {
    User::delete_by_id(id).exec(db).await?;
    Ok(())
}