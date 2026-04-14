use simple_db_query::builders::FilterDefinition;

pub fn compile_filter(filter: &FilterDefinition) -> Box<dyn Fn(&dyn DbRow) -> bool + Send + Sync> {
    
}