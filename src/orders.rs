use chrono::{DateTime, Utc};
use pet::PetCategory;
use pet::PetSize;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::pet;

#[derive(Serialize, Deserialize, sqlx::Type, PartialEq, Debug)]
enum OrderStatus {
    Awaiting,
    Approved,
    Delivered,
    Cancelled,
}

#[derive(Serialize, Deserialize, FromRow, PartialEq, Debug)]
pub struct OrderDetails {
    status: OrderStatus,
    delivered: Option<DateTime<Utc>>,
    details: Option<String>,
}

#[derive(Serialize, Deserialize, FromRow, PartialEq, Debug)]
pub struct Order {
    id: u32,
    pet_id: u32,
    quantity: u32,
    ship_date: DateTime<Utc>,
    details: OrderDetails,
    complete: bool,
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
    use super::{OrderPet, OrderStatus};
    use anyhow::Result;

    fn create_order(req: OrderPet) {}
    fn cancel_order(order_id: u32, reason: &str) -> Result<OrderStatus> {
        Ok(OrderStatus::Cancelled)
    }
    fn check_status(order_id: u32) {}
}

mod storage {
    use std::sync::Arc;

    
    use chrono::{DateTime, Utc};
    use sqlx::{prelude::FromRow, Executor};

    use crate::{persistence::Storage, AppState};

    use super::{Order, OrderDetails, OrderStatus};
    use anyhow::Result;

    pub struct OrderStorage {
        pub storage: Arc<Storage>,
    }

    #[derive(FromRow)]
    struct OrderDB {
        pub id: u32,
        pub pet_id: u32,
        pub quantity: u32,
        pub ship_date: DateTime<Utc>,
        pub status: OrderStatus,
        pub complete: bool,
        pub delivered: Option<DateTime<Utc>>,
        pub details: Option<String>,
    }
    impl OrderStorage {
        fn create(pool: &AppState) -> OrderStorage {
            let storage = pool.inner.clone().db_pool.clone();
            OrderStorage { storage }
        }

        #[tracing::instrument(skip(self))]
        pub async fn create_order(self, order: Order) -> Result<()> {
            sqlx::query(
                "insert into orders (
                id
                pet_id
                quantity
                ship_date
                status
                complete
            )
            values (?,?,?,?,?,?)
            ",
            )
            .bind(order.id)
            .bind(order.pet_id)
            .bind(order.quantity)
            .bind(order.ship_date)
            .bind(&order.details.status)
            .bind(order.complete)
            .execute(&self.storage.conn())
            .await?;

            Ok(())
        }
        #[tracing::instrument(skip(self))]
        pub async fn get_order(self, order_id: u32) -> Result<Option<Order>> {
            let res: Option<OrderDB> = sqlx::query_as(
                "select * 
                from orders o 
                join order_details od on o.id = od.order_id",
            )
            .bind(order_id)
            .fetch_optional(&self.storage.conn())
            .await?;

            res.map_or(Ok(None), |res| {
                Ok(Some(Order {
                    id: res.id,
                    pet_id: res.pet_id,
                    quantity: res.quantity,
                    ship_date: res.ship_date,
                    details: OrderDetails {
                        status: res.status,
                        delivered: res.delivered,
                        details: res.details,
                    },
                    complete: res.complete,
                }))
            })
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
        use std::sync::Arc;

        use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

        use crate::{
            orders::{storage::OrderStorage, Order, OrderDetails, OrderStatus},
            persistence::{Storage, StorageConfig},
        };

        struct TestConnection {
            storage: Arc<Storage>,
        }
        impl TestConnection {
            async fn create() -> anyhow::Result<Self> {
                let storage = Storage::open_migrate(StorageConfig {
                    db_path: "sqlite::memory:".into(),
                })
                .await?;

                Ok(TestConnection {
                    storage: Arc::new(storage),
                })
            }

            async fn close(self) {
                self.storage.close().await
            }
        }

        #[tokio::test]
        async fn get_order() -> anyhow::Result<()> {
            let storage = TestConnection::create().await?;

            let orders_storage = OrderStorage {
                storage: storage.storage.clone(),
            };

            let res = orders_storage.get_order(1).await?;

            assert_eq!(None, res);

            storage.close().await;

            Ok(())
        }
        #[tokio::test]
        async fn insert_order() -> anyhow::Result<()> {
            let storage = TestConnection::create().await?;
            let orders_storage = OrderStorage {
                storage: storage.storage.clone(),
            };

            let d = NaiveDate::from_ymd_opt(2024, 8, 14).unwrap();
            let t = NaiveTime::from_hms_milli_opt(23, 22, 0, 0).unwrap();
            let test_order = Order {
                id: 32,
                pet_id: 32,
                quantity: 32,
                ship_date: NaiveDateTime::new(d, t).and_utc(),

                details: OrderDetails {
                    status: OrderStatus::Approved,
                    delivered: None,
                    details: None,
                },
                complete: false,
            };

            let res = orders_storage.create_order(test_order).await;

            assert!(res.is_ok());

            Ok(())
        }
    }
}
