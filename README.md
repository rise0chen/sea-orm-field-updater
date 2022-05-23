# sea-orm-field-updater

Provides `FieldUpdater` derive macro.

```rust, ignore
#[derive(FieldUpdater)]
#[derive(DeriveEntityModel)]
#[derive(StructField)]
#[derive(Debug, Clone, PartialEq)]
#[sea_orm(table_name = "task")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub finish_at: Option<DateTime>,
}
```

generates

```rust, ignore
impl Model {
    pub fn str2col(s: &str) -> Option<Column> {
        match s {
            "id" => Some(Column::Id),
            "finish_at" => Some(Column::FinishAt),
            _ => None,
        }
    }
    pub fn field2cv(field: ModelField) -> (Column, SimpleExpr) {
        match field {
            ModelField::id(v) => (Column::Id, Expr::value(v)),
            ModelField::finish_at(v) => (Column::FinishAt, Expr::value(v)),
        }
    }
    pub fn fields2active(fields: Vec<ModelField>) -> ActiveModel {
        let mut model = ActiveModel::new();
        for field in fields {
            match field {
                ModelField::id(v) => model.id = Set(v),
                ModelField::finish_at(v) => model.finish_at = Set(v),
            }
        }
        model
    }
}
```
