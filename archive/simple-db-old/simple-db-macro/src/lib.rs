use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Type};

/// Automatically implements DbEntityModel, FromDbRow, and Into<DbRow> for a struct.
///
/// # Struct-Level Attributes
/// - `#[db_entity(collection = "table_name")]` - Required
///   - `collection`: The database table name
///
/// # Field-Level Attributes
/// - `#[db_entity(primary_key)]` - Marks a field as part of the primary key (required at least once)
///   - Can be applied to multiple fields for composite keys
///
/// # Supported Field Types
/// **Primitive types:**
/// - `i8, i16, i32, i64, i128` → `take_i*()`
/// - `u8, u16, u32, u64, u128` → `take_u*()`
/// - `f32, f64` → `take_f*()`
/// - `bool` → `take_bool()`
/// - `char` → `take_char()`
///
/// **Temporal types:**
/// - `chrono::NaiveDate` → `take_date()`
/// - `chrono::NaiveTime` → `take_time()`
/// - `chrono::NaiveDateTime` → `take_timestamp()`
/// - `chrono::DateTime<Utc>` → `take_timestamptz()`
///
/// **Large/boxed types:**
/// - `String` → `take_string()`
/// - `rust_decimal::Decimal` → `take_decimal()`
/// - `uuid::Uuid` → `take_uuid()`
/// - `Vec<u8>` → `take_bytes()`
/// - `serde_json::Value` → `take_json()`
///
/// # Example
/// ```ignore
/// use simple_db::DbEntity;
///
/// #[derive(DbEntity, Clone, Debug)]
/// #[db_entity(collection = "users")]
/// pub struct User {
///     #[db_entity(primary_key)]
///     pub id: i32,
///     pub username: String,
///     pub email: String,
/// }
///
/// // Composite key example:
/// #[derive(DbEntity, Clone, Debug)]
/// #[db_entity(collection = "user_roles")]
/// pub struct UserRole {
///     #[db_entity(primary_key)]
///     pub user_id: i32,
///     #[db_entity(primary_key)]
///     pub role_id: i32,
///     pub assigned_at: String,
/// }
/// ```
#[proc_macro_derive(DbEntity, attributes(db_entity))]
pub fn derive_db_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(&input, "DbEntity only supports named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "DbEntity only supports structs")
                .to_compile_error()
                .into();
        }
    };

    // Extract struct-level attributes
    let mut collection_name = String::new();

    for attr in &input.attrs {
        if attr.path().is_ident("db_entity") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("collection") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    collection_name = value.value();
                }
                Ok(())
            });
        }
    }

    if collection_name.is_empty() {
        return syn::Error::new_spanned(
            &input,
            "DbEntity requires #[db_entity(collection = \"table_name\")]",
        )
        .to_compile_error()
        .into();
    }

    // Build field access info and detect primary keys
    let mut field_info = Vec::new();
    let mut key_fields = Vec::new();

    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let field_name = ident.to_string();
        let read_method = infer_read_method(ty);

        // Check for primary_key attribute
        let mut is_pk = false;
        for attr in &field.attrs {
            if attr.path().is_ident("db_entity") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("primary_key") {
                        is_pk = true;
                    }
                    Ok(())
                });
            }
        }

        if is_pk {
            key_fields.push((ident.clone(), field_name.clone(), ty.clone()));
        }

        field_info.push((ident.clone(), field_name, read_method, ty.clone()));
    }

    if key_fields.is_empty() {
        return syn::Error::new_spanned(
            &input,
            "DbEntity requires at least one field marked with #[db_entity(primary_key)]",
        )
        .to_compile_error()
        .into();
    }

    // Generate field read code for FromDbRow
    let field_reads = field_info.iter().map(|(ident, field_name, method, _ty)| {
        let method_ident = syn::Ident::new(method, proc_macro2::Span::call_site());
        quote! {
            #ident: row.#method_ident(#field_name)?,
        }
    });

    // Generate field write code for Into<DbRow>
    //
    // Important: `Into<DbRow>` consumes `self`, so we can move non-Copy fields
    // (String, Vec, Decimal, etc.) without cloning.
    let field_writes = field_info.iter().map(|(ident, _field_name, _method, ty)| {
        if is_copy_type(ty) {
            quote! {
                row.insert(stringify!(#ident), #ident);
            }
        } else {
            quote! {
                row.insert(stringify!(#ident), #ident);
            }
        }
    });

    let field_idents = field_info.iter().map(|(ident, _field_name, _method, _ty)| ident);

    // Generate key() method
    let key_reads = key_fields.iter().map(|(key_ident, key_str, ty)| {
        if is_copy_type(ty) {
            quote! {
                (#key_str.to_string(), ::simple_db::types::DbValue::from(self.#key_ident)),
            }
        } else {
            quote! {
                (#key_str.to_string(), ::simple_db::types::DbValue::from(self.#key_ident.clone())),
            }
        }
    });

    let expanded = quote! {
        impl ::simple_db::types::FromDbRow for #name {
            fn from_db_row(row: &mut ::simple_db::types::DbRow) -> Result<Self, ::simple_db::types::DbError> {
                Ok(Self {
                    #(#field_reads)*
                })
            }
        }

        impl Into<::simple_db::types::DbRow> for #name {
            fn into(self) -> ::simple_db::types::DbRow {
                let Self { #(#field_idents),* } = self;
                let mut row = ::simple_db::types::DbRow::new();
                #(#field_writes)*
                row
            }
        }

        impl ::simple_db::entity::DbEntityModel for #name {
            fn collection_name() -> &'static str {
                #collection_name
            }

            fn key(&self) -> ::simple_db::entity::DbEntityKey {
                vec![
                    #(#key_reads)*
                ]
            }
        }
    };

    TokenStream::from(expanded)
}

/// Infer the correct field read method based on field type
fn infer_read_method(ty: &Type) -> &'static str {
    match ty {
        Type::Path(type_path) => {
            let last_segment = type_path.path.segments.last();
            match last_segment {
                Some(seg) => {
                    let type_name = seg.ident.to_string();
                    match type_name.as_str() {
                        // Primitive types - stack values
                        "i8" => "take_i8",
                        "i16" => "take_i16",
                        "i32" => "take_i32",
                        "i64" => "take_i64",
                        "i128" => "take_i128",
                        "u8" => "take_u8",
                        "u16" => "take_u16",
                        "u32" => "take_u32",
                        "u64" => "take_u64",
                        "u128" => "take_u128",
                        "f32" => "take_f32",
                        "f64" => "take_f64",
                        "bool" => "take_bool",
                        "char" => "take_char",
                        
                        // Temporal types (from chrono)
                        "NaiveDate" => "take_date",
                        "NaiveTime" => "take_time",
                        "NaiveDateTime" => "take_timestamp",
                        "DateTime" => "take_timestamptz",
                        
                        // Large/boxed types
                        "String" => "take_string",
                        "Decimal" => "take_decimal",
                        "Uuid" => "take_uuid",
                        "Vec" => "take_bytes",    // Vec<u8>
                        "Value" => "take_json",   // serde_json::Value
                        
                        // Default fallback
                        _ => "take_string",
                    }
                }
                None => "take_string",
            }
        }
        _ => "take_string",
    }
}

/// Determines if a type is Copy and doesn't need to be cloned
/// when converting to DbRow or extracting for keys.
/// 
/// Copy types: all primitives, small stack-allocated types like Uuid
/// Non-Copy types: String, Decimal, Vec, serde_json::Value
fn is_copy_type(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => {
            let last_segment = type_path.path.segments.last();
            match last_segment {
                Some(seg) => {
                    let type_name = seg.ident.to_string();
                    matches!(
                        type_name.as_str(),
                        // Primitive types (all are Copy)
                        "i8" | "i16" | "i32" | "i64" | "i128" |
                        "u8" | "u16" | "u32" | "u64" | "u128" |
                        "f32" | "f64" | "bool" | "char" |
                        // Temporal types from chrono (all are Copy)
                        "NaiveDate" | "NaiveTime" | "NaiveDateTime" | "DateTime" |
                        // Uuid is Copy
                        "Uuid"
                    )
                }
                None => false,
            }
        }
        _ => false,
    }
}
