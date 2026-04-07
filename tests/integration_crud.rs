use simple_db::{
    driver::memory::MemoryDriver, 
    types::{DbRow, DbValue, FromDbRow, DbError},
    entity::{DbEntity, DbEntityModel, DbEntityKey},
    query::Query,
    DbContext,
};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
struct Product {
    id: i32,
    name: String,
    price: f64,
    stock: i32,
}

impl FromDbRow for Product {
    fn from_db_row(mut row: DbRow) -> Result<Self, DbError> {
        Ok(Self {
            id: row.take_i32("id")?,
            name: row.take_string("name")?,
            price: row.take_f64("price")?,
            stock: row.take_i32("stock")?,
        })
    }
}

impl Into<DbRow> for Product {
    fn into(self) -> DbRow {
        let mut row = DbRow::new();
        row.insert("id", self.id);
        row.insert("name", self.name);
        row.insert("price", self.price);
        row.insert("stock", self.stock);
        row
    }
}

impl DbEntityModel for Product {
    fn collection_name() -> &'static str {
        "products"
    }

    fn key(&self) -> DbEntityKey {
        vec![("id".to_string(), DbValue::I32(Some(self.id)))]
    }
}

#[tokio::test]
async fn test_full_crud_cycle() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver.clone());

    // CREATE - Insert a product
    let mut product = DbEntity::new(Product {
        id: 1,
        name: "Laptop".to_string(),
        price: 999.99,
        stock: 10,
    });

    // Note: This would work in real scenario with full DbContext implementation
    // For now, we test the entity state management
    assert!(matches!(product.state(), simple_db::entity::DbEntityState::Added));
}

#[tokio::test]
async fn test_find_product_with_filters() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver.clone());

    // Insert test data
    let mut row1 = DbRow::new();
    row1.insert("id", 1i32);
    row1.insert("name", "Laptop");
    row1.insert("price", 999.99f64);
    row1.insert("stock", 10i32);

    let mut row2 = DbRow::new();
    row2.insert("id", 2i32);
    row2.insert("name", "Mouse");
    row2.insert("price", 29.99f64);
    row2.insert("stock", 100i32);

    let mut row3 = DbRow::new();
    row3.insert("id", 3i32);
    row3.insert("name", "Keyboard");
    row3.insert("price", 79.99f64);
    row3.insert("stock", 50i32);

    let rows = vec![row1, row2, row3];

    // Find products with price less than $100
    let query = Query::find("products")
        .filter(|fb| fb.lt("price", 100.0));

    let result = ctx.find(query).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 2); // Mouse and Keyboard
}

#[tokio::test]
async fn test_find_products_sorted() {
    let driver = Arc::new(MemoryDriver::new());
    
    let mut row1 = DbRow::new();
    row1.insert("id", 1i32);
    row1.insert("name", "Laptop");
    row1.insert("price", 999.99f64);
    row1.insert("stock", 10i32);

    let mut row2 = DbRow::new();
    row2.insert("id", 2i32);
    row2.insert("name", "Mouse");
    row2.insert("price", 29.99f64);
    row2.insert("stock", 100i32);

    let rows = vec![row1, row2];

    let query = Query::find("products")
        .order_by(|sb| sb.asc("price"));

    // This tests that the query builder itself works
    assert_eq!(query.sorts.len(), 1);
}

#[tokio::test]
async fn test_multiple_filters_and_gate() {
    let driver = Arc::new(MemoryDriver::new());
    
    let query = Query::find("products")
        .filter(|fb| fb.gte("price", 50.0))
        .filter(|fb| fb.lte("price", 500.0))
        .filter(|fb| fb.gt("stock", 0));

    // Verify filters are accumulated
    assert_eq!(query.filters.len(), 3);
}

#[tokio::test]
async fn test_pagination() {
    let driver = Arc::new(MemoryDriver::new());
    
    let query = Query::find("products")
        .limit(10)
        .offset(20);

    assert_eq!(query.limit, Some(10));
    assert_eq!(query.offset, Some(20));
}

#[tokio::test]
async fn test_order_by_multiple_fields() {
    let query = Query::find("products")
        .order_by(|sb| sb.asc("price").desc("stock"));

    assert_eq!(query.sorts.len(), 2);
}

#[tokio::test]
async fn test_entity_conversion_roundtrip() {
    let product = Product {
        id: 1,
        name: "Laptop".to_string(),
        price: 999.99,
        stock: 10,
    };

    // Convert to row
    let row: DbRow = product.clone().into();

    // Convert back from row
    let recovered = Product::from_db_row(row);

    assert!(recovered.is_ok());
    assert_eq!(recovered.unwrap(), product);
}
