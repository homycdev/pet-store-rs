use chrono::{DateTime, Utc};
use pet::PetCategory;
use pet::PetSize;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::pet;

#[derive(Serialize, Deserialize, sqlx::Type, PartialEq, Debug, Clone)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
enum OrderStatus {
    Awaiting,
    Approved,
    Delivered,
    Cancelled,
}

#[derive(Serialize, Deserialize, FromRow, PartialEq, Debug, Clone)]
pub struct OrderDetails {
    status: OrderStatus,
    delivered: Option<DateTime<Utc>>,
    details: Option<String>,
}

#[derive(Serialize, Deserialize, FromRow, PartialEq, Debug, Clone)]
pub struct Order {
    id: u64,
    user_id: u64,
    pet_id: u64,
    quantity: u64,
    ship_date: Option<DateTime<Utc>>,
    details: OrderDetails,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct OrderPet {
    category: PetCategory,
    pet_size: PetSize,
}

impl OrderPet {
    pub fn new(category: PetCategory, pet_size: PetSize) -> Self {
        Self { category, pet_size }
    }
}

mod service {
    // use super::{OrderPet, OrderStatus};
    // use anyhow::Result;

    // fn create_order(req: OrderPet) {}
    // fn cancel_order(order_id: u32, reason: &str) -> Result<OrderStatus> {
    //     Ok(OrderStatus::Cancelled)
    // }
    // fn check_status(order_id: u32) {}
}

mod storage {
    

    use chrono::{DateTime, Utc};
    use sqlx::{FromRow, Postgres};

    use crate::persistence::ArcPgPool;

    use super::OrderStatus;
    use anyhow::Result;

    pub struct OrderStorage {
        pub storage: ArcPgPool,
    }

    #[derive(Clone, FromRow, PartialEq, Debug)]
    pub struct OrderDB {
        pub id: i64,
        pub pet_id: i64,
        pub user_id: i64,
        pub quantity: i64,
        pub ship_date: Option<DateTime<Utc>>,
        pub status: OrderStatus,
    }
    impl OrderStorage {
        #[tracing::instrument(skip(self))]
        pub async fn create(&self, order: OrderDB) -> Result<()> {
            sqlx::query::<Postgres>("insert into orders ( id, pet_id, user_id, quantity, ship_date, status) values ($1,$2,$3,$4,$5,$6);")
                .bind(order.id)
                .bind(order.pet_id)
                .bind(order.user_id)
                .bind(order.quantity)
                .bind(order.ship_date)
                .bind(order.status)
                .execute(&self.storage.clone())
                .await?;

            Ok(())
        }
        #[tracing::instrument(skip(self))]
        pub async fn get(&self, order_id: u64) -> Result<Option<OrderDB>> {
            let res: Option<OrderDB> = sqlx::query_as(
                "select * 
                from orders o 
                where o.id = $1",
            )
            .bind(order_id as i64)
            .fetch_optional(&self.storage.clone())
            .await?;

            Ok(res)
        }

        pub async fn delete(&self, id: i64) -> Result<()> {
            let _ = sqlx::query("delete from orders where id = $1")
                .bind(id)
                .execute(&self.storage.clone())
                .await?;
            Ok(())
        }

        pub async fn update(&self, o: OrderDB) -> Result<()> {
            let _ = sqlx::query::<Postgres>(
                "insert into orders as o(id, pet_id, user_id, quantity, ship_date, status) 
                values ($1, $2, $3, $4, $5, $6)
                on conflict(id) 
                do update set 
                    pet_id = excluded.pet_id, 
                    user_id = excluded.user_id, 
                    quantity = excluded.quantity, 
                    ship_date = excluded.ship_date, 
                    status = excluded.status;
                ",
            )
            .bind(o.id)
            .bind(o.pet_id)
            .bind(o.user_id)
            .bind(o.quantity)
            .bind(o.ship_date)
            .bind(o.status)
            .execute(&self.storage.clone())
            .await?;

            Ok(())
        }
    }
}

mod api {
    // use axum::{http::uri, Router};
    // use serde::Deserialize;
    // use tower::Service;

    // #[derive(Deserialize)]
    // pub struct QueryOrderId {
    //     pub order_id: u32,
    // }
    // const ORDER_URL: &str = "/store/order";
    // const ORDER_CANCEL_URL: &str = "/store/order/cancel";
}

#[cfg(test)]
mod tests {

    mod storage {

        use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

        use crate::{
            config::AppConfig,
            orders::{
                storage::{OrderDB, OrderStorage}, OrderStatus,
            },
            persistence::{Storage, StorageConfig, StorageI},
        };

        async fn fixture() -> anyhow::Result<OrderStorage> {
            let config = AppConfig::load_config()?;

            let storage_conf = StorageConfig {
                db_path: config.db(),
            };
            let storage = StorageI.conn(storage_conf).await?;
            StorageI.migrate(storage.pool.clone()).await?;

            Ok(OrderStorage {
                storage: storage.pool.clone(),
            })
        }

        #[tokio::test]
        async fn get_order() -> anyhow::Result<()> {
            let orders_storage = fixture().await?;

            let res = orders_storage.get(0).await?;

            assert_eq!(None, res);

            orders_storage.storage.close().await;

            Ok(())
        }
        #[tokio::test]
        async fn insert_order() -> anyhow::Result<()> {
            let orders_storage = fixture().await?;

            let d = NaiveDate::from_ymd_opt(2024, 8, 14).unwrap();
            let t = NaiveTime::from_hms_milli_opt(23, 22, 0, 0).unwrap();
            let test_order = OrderDB {
                id: 1,
                pet_id: 0,
                user_id: 0,
                quantity: 32,
                ship_date: Some(NaiveDateTime::new(d, t).and_utc()),
                status: OrderStatus::Approved,
            };
            orders_storage.create(test_order.clone()).await?;
            let get_res = orders_storage.get(1).await?;

            assert_eq!((test_order.id) as i64, get_res.unwrap().id);
            orders_storage.delete(1).await?;

            orders_storage.storage.close().await;

            Ok(())
        }
        #[tokio::test]
        async fn update_order() -> anyhow::Result<()> {
            let order_storage = fixture().await?;
            let d = NaiveDate::from_ymd_opt(2024, 8, 14).unwrap();
            let t = NaiveTime::from_hms_milli_opt(23, 22, 0, 0).unwrap();

            let test_order = OrderDB {
                id: 1,
                pet_id: 0,
                user_id: 0,
                quantity: 32,
                ship_date: Some(NaiveDateTime::new(d.clone(), t.clone()).and_utc()),
                status: OrderStatus::Approved,
            };

            order_storage.create(test_order).await?;
            order_storage
                .update(OrderDB {
                    id: 1,
                    pet_id: 0,
                    user_id: 1,
                    quantity: 0,
                    ship_date: Some(NaiveDateTime::new(d.clone(), t.clone()).and_utc()),
                    status: OrderStatus::Cancelled,
                })
                .await?;

            let result = order_storage.get(1).await?;

            assert_eq!(
                Some(OrderDB {
                    id: 1,
                    pet_id: 0,
                    user_id: 1,
                    quantity: 0,
                    ship_date: Some(NaiveDateTime::new(d.clone(), t.clone()).and_utc()),
                    status: OrderStatus::Cancelled,
                }),
                result
            );

            order_storage.delete(1).await?;

            Ok(())
        }
    }
}
