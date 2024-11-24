use crate::{
    entity::{prelude::User, user},
    model::user::{CreateUserReq, UpdateUserReq},
};
use sea_orm::*;

pub async fn create(user: CreateUserReq, db: &DatabaseConnection) -> Result<user::Model, DbErr> {
    User::insert(user::ActiveModel {
        email: Set(user.email),
        name: Set(user.name),
        password: Set(bcrypt::hash(user.password, bcrypt::DEFAULT_COST)
            .map_err(|err| DbErr::Custom(err.to_string()))?),
        ..Default::default()
    })
    .exec_with_returning(db)
    .await
}

pub async fn find_by_id(id: String, db: &DatabaseConnection) -> Result<Option<user::Model>, DbErr> {
    User::find_by_id(id).one(db).await
}

pub async fn find_by_email(
    email: &String,
    db: &DatabaseConnection,
) -> Result<Option<user::Model>, DbErr> {
    User::find()
        .filter(user::Column::Email.eq(email))
        .one(db)
        .await
}

pub async fn update(data: UpdateUserReq, db: &DatabaseConnection) -> Result<user::Model, DbErr> {
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
        update.password = Set(bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|err| DbErr::Custom(err.to_string()))?);
    }

    update.save(db).await?.try_into_model()
}

pub async fn delete(id: String, db: &DatabaseConnection) -> Result<(), DbErr> {
    User::delete_by_id(id).exec(db).await?;
    Ok(())
}
