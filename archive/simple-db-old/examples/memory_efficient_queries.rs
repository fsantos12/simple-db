//! Memory-Efficient Query Patterns
//!
//! This example demonstrates when to use `find_entities_readonly` vs `find_entities`.
//!
//! Key insight: Use `find_entities_readonly` for read-only queries to save 50% memory
//! (no snapshot overhead) and get ~2× speedup for large result sets.

use simple_db::{DbContext, DbEntity, DbEntityModel, query::Query};

#[derive(Clone, Debug)]
struct User {
    id: i32,
    name: String,
    email: String,
    active: bool,
}

impl DbEntityModel for User {
    fn collection_name() -> &'static str {
        "users"
    }

    fn key(&self) -> Vec<(String, simple_db::types::DbValue)> {
        vec![("id".to_string(), simple_db::types::DbValue::from(self.id))]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create database context (simplified - would need real DB initialization)
    let ctx = DbContext::new();

    println!("=== Memory-Efficient Query Patterns ===\n");

    // =========================================================================
    // PATTERN 1: Read-Only Display (BEST CASE for find_entities_readonly)
    // =========================================================================
    println!("📊 Pattern 1: Read-Only Display\n");
    println!("Scenario: Fetch users just to display in a web response");
    println!("Memory: O(n) | Speed: Fast | Use: find_entities_readonly ✅\n");

    // ❌ NOT RECOMMENDED
    println!("  ❌ WRONG: Using find_entities for read-only display");
    println!("     let users: Vec<DbEntity<User>> = ctx.find_entities(");
    println!("         Query::find(\"users\").limit(100)");
    println!("     ).await?;");
    println!("     for user in users {{");
    println!("         println!(\"User: {{}}\", user.entity.name);");
    println!("     }}\n");
    println!("     Issue: Each entity has a snapshot (2× memory) but never uses it!\n");

    // ✅ RECOMMENDED
    println!("  ✅ CORRECT: Using find_entities_readonly for read-only display");
    println!("     let users: Vec<User> = ctx.find_entities_readonly::<User>(");
    println!("         Query::find(\"users\").limit(100)");
    println!("     ).await?;");
    println!("     for user in users {{");
    println!("         println!(\"User: {{}}\", user.name);");
    println!("     }}\n");
    println!("     Benefit: No snapshot overhead, direct entity access, 2× faster!\n");

    // =========================================================================
    // PATTERN 2: Data Export / Aggregation
    // =========================================================================
    println!("📤 Pattern 2: Data Export / Aggregation\n");
    println!("Scenario: Export user data to CSV or compute statistics");
    println!("Memory: O(n) | Speed: Very Fast | Use: find_entities_readonly ✅\n");

    println!("  ❌ WRONG: Using find_entities for statistics");
    println!("     let users = ctx.find_entities::<User>(");
    println!("         Query::find(\"users\")");
    println!("     ).await?;");
    println!("     let total = users.len();");
    println!("     let active_count = users.iter().filter(|u| u.entity.active).count();\n");
    println!("     Problem: Loads 2×n data but only reads from it\n");

    println!("  ✅ CORRECT: Using find_entities_readonly for statistics");
    println!("     let users: Vec<User> = ctx.find_entities_readonly::<User>(");
    println!("         Query::find(\"users\")");
    println!("     ).await?;");
    println!("     let total = users.len();");
    println!("     let active_count = users.iter().filter(|u| u.active).count();\n");
    println!("     Benefit: O(n) memory, no unnecessary data duplication!\n");

    // =========================================================================
    // PATTERN 3: Modify Records (BEST CASE for find_entities)
    // =========================================================================
    println!("✏️  Pattern 3: Modify Records\n");
    println!("Scenario: Fetch users and update some fields");
    println!("Memory: O(2n) | Speed: Slower (tracking) | Use: find_entities ❌→✅\n");

    println!("  ✅ CORRECT: Using find_entities when you'll modify");
    println!("     let mut users = ctx.find_entities::<User>(");
    println!("         Query::find(\"users\")");
    println!("     ).await?;");
    println!("     for mut user in users {{");
    println!("         user.entity.active = true;");
    println!("         user.save(&ctx).await?;  // ← Can call save()");
    println!("     }}\n");
    println!("     Rationale: Snapshot needed to track changes for UPDATE queries\n");

    println!("  ❌ WRONG: Using find_entities_readonly when you'll modify");
    println!("     let users: Vec<User> = ctx.find_entities_readonly::<User>(");
    println!("         Query::find(\"users\")");
    println!("     ).await?;");
    println!("     for mut user in users {{");
    println!("         user.active = true;  // Changes are lost!");
    println!("         // user.save(&ctx).await?;  // ← This method doesn't exist!");
    println!("     }}\n");
    println!("     Problem: No way to persist changes!\n");

    // =========================================================================
    // MEMORY IMPACT VISUALIZATION
    // =========================================================================
    println!("📊 Memory Impact\n");
    println!("Query returning 10,000 rows:");
    println!("  Method                      | Memory Used   | Difference");
    println!("  ─────────────────────────────────────────────────────────");
    println!("  find_entities_readonly      | ~10MB (1×)    | ← EFFICIENT");
    println!("  find_entities               | ~20MB (2×)    | +100% overhead\n");

    println!("Query returning 100,000 rows:");
    println!("  Method                      | Memory Used   | Difference");
    println!("  ─────────────────────────────────────────────────────────");
    println!("  find_entities_readonly      | ~100MB (1×)   | ← EFFICIENT");
    println!("  find_entities               | ~200MB (2×)   | +100MB wasted\n");

    // =========================================================================
    // DECISION TREE
    // =========================================================================
    println!("🌳 Quick Decision Tree\n");
    println!("1. Do you need to call .save() on entities?");
    println!("   → YES: Use find_entities");
    println!("   → NO:  Use find_entities_readonly ✅ (2× faster, 50% less memory)\n");

    println!("2. Will you modify entity fields?");
    println!("   → YES: Use find_entities");
    println!("   → NO:  Use find_entities_readonly ✅\n");

    println!("3. Is this for display/export/reporting only?");
    println!("   → YES: Use find_entities_readonly ✅ (BEST CHOICE)");
    println!("   → NO:  Use find_entities\n");

    Ok(())
}
