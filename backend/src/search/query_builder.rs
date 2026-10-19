use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Search query types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QueryType {
    Match {
        field: String,
        query: String,
    },
    MultiMatch {
        fields: Vec<String>,
        query: String,
    },
    Term {
        field: String,
        value: Value,
    },
    Range {
        field: String,
        gte: Option<Value>,
        lte: Option<Value>,
        gt: Option<Value>,
        lt: Option<Value>,
    },
    Bool {
        must: Vec<QueryType>,
        should: Vec<QueryType>,
        must_not: Vec<QueryType>,
    },
    Wildcard {
        field: String,
        value: String,
    },
    Prefix {
        field: String,
        value: String,
    },
    Fuzzy {
        field: String,
        value: String,
        fuzziness: Option<String>,
    },
    MatchAll,
}

/// Main search query structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: QueryType,
    pub from: usize,
    pub size: usize,
    pub sort: Option<Vec<Value>>,
    pub highlight: Option<Value>,
    pub aggregations: Option<HashMap<String, Value>>,
    #[serde(rename = "_source")]
    pub source: Option<Vec<String>>,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            query: QueryType::MatchAll,
            from: 0,
            size: 20,
            sort: None,
            highlight: None,
            aggregations: None,
            source: None,
        }
    }
}

/// Builder for constructing search queries
pub struct SearchQueryBuilder {
    query: SearchQuery,
}

impl SearchQueryBuilder {
    pub fn new() -> Self {
        Self {
            query: SearchQuery::default(),
        }
    }

    /// Set the main query
    pub fn query(mut self, query: QueryType) -> Self {
        self.query.query = query;
        self
    }

    /// Match query on a single field
    pub fn match_query(mut self, field: impl Into<String>, query: impl Into<String>) -> Self {
        self.query.query = QueryType::Match {
            field: field.into(),
            query: query.into(),
        };
        self
    }

    /// Multi-match query across multiple fields
    pub fn multi_match(mut self, fields: Vec<String>, query: impl Into<String>) -> Self {
        self.query.query = QueryType::MultiMatch {
            fields,
            query: query.into(),
        };
        self
    }

    /// Term query for exact matches
    pub fn term(mut self, field: impl Into<String>, value: Value) -> Self {
        self.query.query = QueryType::Term {
            field: field.into(),
            value,
        };
        self
    }

    /// Boolean query combining multiple conditions
    pub fn bool_query(mut self, must: Vec<QueryType>, should: Vec<QueryType>, must_not: Vec<QueryType>) -> Self {
        self.query.query = QueryType::Bool { must, should, must_not };
        self
    }

    /// Set pagination offset
    pub fn from(mut self, from: usize) -> Self {
        self.query.from = from;
        self
    }

    /// Set page size
    pub fn size(mut self, size: usize) -> Self {
        self.query.size = size;
        self
    }

    /// Add sorting
    pub fn sort(mut self, field: impl Into<String>, order: impl Into<String>) -> Self {
        let sort_obj = json!({ field.into(): { "order": order.into() } });

        match &mut self.query.sort {
            Some(sorts) => sorts.push(sort_obj),
            None => self.query.sort = Some(vec![sort_obj]),
        }

        self
    }

    /// Add highlighting
    pub fn highlight(mut self, fields: Vec<String>) -> Self {
        let mut highlight_fields = HashMap::new();
        for field in fields {
            highlight_fields.insert(field, json!({}));
        }

        self.query.highlight = Some(json!({
            "fields": highlight_fields,
            "pre_tags": ["<em>"],
            "post_tags": ["</em>"],
        }));

        self
    }

    /// Add aggregation
    pub fn aggregation(mut self, name: impl Into<String>, agg: Value) -> Self {
        match &mut self.query.aggregations {
            Some(aggs) => {
                aggs.insert(name.into(), agg);
            }
            None => {
                let mut aggs = HashMap::new();
                aggs.insert(name.into(), agg);
                self.query.aggregations = Some(aggs);
            }
        }

        self
    }

    /// Specify which fields to return
    pub fn source(mut self, fields: Vec<String>) -> Self {
        self.query.source = Some(fields);
        self
    }

    /// Build the final query
    pub fn build(self) -> SearchQuery {
        self.query
    }

    /// Convert to Elasticsearch JSON
    pub fn to_elasticsearch_json(&self) -> Value {
        let mut body = json!({
            "from": self.query.from,
            "size": self.query.size,
        });

        // Add query
        body["query"] = self.query_type_to_json(&self.query.query);

        // Add optional fields
        if let Some(ref sort) = self.query.sort {
            body["sort"] = json!(sort);
        }

        if let Some(ref highlight) = self.query.highlight {
            body["highlight"] = highlight.clone();
        }

        if let Some(ref aggs) = self.query.aggregations {
            body["aggs"] = json!(aggs);
        }

        if let Some(ref source) = self.query.source {
            body["_source"] = json!(source);
        }

        body
    }

    fn query_type_to_json(&self, query_type: &QueryType) -> Value {
        match query_type {
            QueryType::MatchAll => json!({ "match_all": {} }),
            QueryType::Match { field, query } => json!({
                "match": { field: query }
            }),
            QueryType::MultiMatch { fields, query } => json!({
                "multi_match": {
                    "query": query,
                    "fields": fields
                }
            }),
            QueryType::Term { field, value } => json!({
                "term": { field: value }
            }),
            QueryType::Range {
                field,
                gte,
                lte,
                gt,
                lt,
            } => {
                let mut range = serde_json::Map::new();
                if let Some(v) = gte {
                    range.insert("gte".to_string(), v.clone());
                }
                if let Some(v) = lte {
                    range.insert("lte".to_string(), v.clone());
                }
                if let Some(v) = gt {
                    range.insert("gt".to_string(), v.clone());
                }
                if let Some(v) = lt {
                    range.insert("lt".to_string(), v.clone());
                }
                json!({ "range": { field: range } })
            }
            QueryType::Bool { must, should, must_not } => {
                let mut bool_query = serde_json::Map::new();

                if !must.is_empty() {
                    let must_queries: Vec<Value> = must.iter().map(|q| self.query_type_to_json(q)).collect();
                    bool_query.insert("must".to_string(), json!(must_queries));
                }

                if !should.is_empty() {
                    let should_queries: Vec<Value> = should.iter().map(|q| self.query_type_to_json(q)).collect();
                    bool_query.insert("should".to_string(), json!(should_queries));
                }

                if !must_not.is_empty() {
                    let must_not_queries: Vec<Value> = must_not.iter().map(|q| self.query_type_to_json(q)).collect();
                    bool_query.insert("must_not".to_string(), json!(must_not_queries));
                }

                json!({ "bool": bool_query })
            }
            QueryType::Wildcard { field, value } => json!({
                "wildcard": { field: value }
            }),
            QueryType::Prefix { field, value } => json!({
                "prefix": { field: value }
            }),
            QueryType::Fuzzy {
                field,
                value,
                fuzziness,
            } => {
                let mut fuzzy = serde_json::Map::new();
                fuzzy.insert("value".to_string(), json!(value));
                if let Some(f) = fuzziness {
                    fuzzy.insert("fuzziness".to_string(), json!(f));
                }
                json!({ "fuzzy": { field: fuzzy } })
            }
        }
    }
}

impl Default for SearchQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_query() {
        let query = SearchQueryBuilder::new().match_query("title", "test").build();

        match query.query {
            QueryType::Match { field, query } => {
                assert_eq!(field, "title");
                assert_eq!(query, "test");
            }
            _ => panic!("Expected Match query"),
        }
    }

    #[test]
    fn test_pagination() {
        let query = SearchQueryBuilder::new().from(10).size(50).build();

        assert_eq!(query.from, 10);
        assert_eq!(query.size, 50);
    }

    #[test]
    fn test_bool_query() {
        let query = SearchQueryBuilder::new()
            .bool_query(
                vec![QueryType::Term {
                    field: "status".to_string(),
                    value: json!("active"),
                }],
                vec![],
                vec![],
            )
            .build();

        match query.query {
            QueryType::Bool { must, .. } => {
                assert_eq!(must.len(), 1);
            }
            _ => panic!("Expected Bool query"),
        }
    }

    // ── #1160: filter query types ────────────────────────────────────────

    #[test]
    fn test_multi_match_query_json() {
        let builder = SearchQueryBuilder::new().multi_match(vec!["title".to_string(), "body".to_string()], "recycling");
        assert_eq!(
            builder.to_elasticsearch_json()["query"],
            json!({ "multi_match": { "query": "recycling", "fields": ["title", "body"] } })
        );
    }

    #[test]
    fn test_term_query_json() {
        let builder = SearchQueryBuilder::new().term("status", json!("active"));
        assert_eq!(
            builder.to_elasticsearch_json()["query"],
            json!({ "term": { "status": "active" } })
        );
    }

    #[test]
    fn test_range_query_json_with_all_bounds() {
        let builder = SearchQueryBuilder::new().query(QueryType::Range {
            field: "created_at".to_string(),
            gte: Some(json!("2024-01-01")),
            lte: Some(json!("2024-12-31")),
            gt: None,
            lt: None,
        });
        assert_eq!(
            builder.to_elasticsearch_json()["query"],
            json!({ "range": { "created_at": { "gte": "2024-01-01", "lte": "2024-12-31" } } })
        );
    }

    #[test]
    fn test_range_query_json_omits_unset_bounds() {
        let builder = SearchQueryBuilder::new().query(QueryType::Range {
            field: "score".to_string(),
            gte: None,
            lte: None,
            gt: Some(json!(1)),
            lt: None,
        });
        assert_eq!(
            builder.to_elasticsearch_json()["query"],
            json!({ "range": { "score": { "gt": 1 } } })
        );
    }

    #[test]
    fn test_wildcard_query_json() {
        let builder = SearchQueryBuilder::new().query(QueryType::Wildcard {
            field: "sku".to_string(),
            value: "PLA-*".to_string(),
        });
        assert_eq!(
            builder.to_elasticsearch_json()["query"],
            json!({ "wildcard": { "sku": "PLA-*" } })
        );
    }

    #[test]
    fn test_prefix_query_json() {
        let builder = SearchQueryBuilder::new().query(QueryType::Prefix {
            field: "category".to_string(),
            value: "plas".to_string(),
        });
        assert_eq!(
            builder.to_elasticsearch_json()["query"],
            json!({ "prefix": { "category": "plas" } })
        );
    }

    #[test]
    fn test_fuzzy_query_json_with_fuzziness() {
        let builder = SearchQueryBuilder::new().query(QueryType::Fuzzy {
            field: "name".to_string(),
            value: "plastik".to_string(),
            fuzziness: Some("AUTO".to_string()),
        });
        assert_eq!(
            builder.to_elasticsearch_json()["query"],
            json!({ "fuzzy": { "name": { "value": "plastik", "fuzziness": "AUTO" } } })
        );
    }

    #[test]
    fn test_fuzzy_query_json_without_fuzziness() {
        let builder = SearchQueryBuilder::new().query(QueryType::Fuzzy {
            field: "name".to_string(),
            value: "plastik".to_string(),
            fuzziness: None,
        });
        assert_eq!(
            builder.to_elasticsearch_json()["query"],
            json!({ "fuzzy": { "name": { "value": "plastik" } } })
        );
    }

    #[test]
    fn test_bool_query_json_combines_must_should_must_not() {
        let builder = SearchQueryBuilder::new().bool_query(
            vec![QueryType::Term {
                field: "status".to_string(),
                value: json!("active"),
            }],
            vec![QueryType::Match {
                field: "title".to_string(),
                query: "urgent".to_string(),
            }],
            vec![QueryType::Term {
                field: "archived".to_string(),
                value: json!(true),
            }],
        );
        let body = builder.to_elasticsearch_json();
        assert!(body["query"]["bool"]["must"].is_array());
        assert!(body["query"]["bool"]["should"].is_array());
        assert!(body["query"]["bool"]["must_not"].is_array());
    }

    #[test]
    fn test_empty_bool_query_omits_empty_clauses() {
        let builder = SearchQueryBuilder::new().bool_query(vec![], vec![], vec![]);
        let body = builder.to_elasticsearch_json();
        assert_eq!(body["query"], json!({ "bool": {} }));
    }

    #[test]
    fn test_nested_bool_query() {
        let inner = QueryType::Bool {
            must: vec![QueryType::Term {
                field: "type".to_string(),
                value: json!("plastic"),
            }],
            should: vec![],
            must_not: vec![],
        };
        let builder = SearchQueryBuilder::new().bool_query(vec![inner], vec![], vec![]);
        let body = builder.to_elasticsearch_json();
        let must = body["query"]["bool"]["must"].as_array().unwrap();
        assert_eq!(must.len(), 1);
        assert!(must[0]["bool"]["must"].is_array());
    }

    // ── #1160: sorting ───────────────────────────────────────────────────

    #[test]
    fn test_single_sort_field() {
        let query = SearchQueryBuilder::new().sort("created_at", "desc").build();
        assert_eq!(
            query.sort.unwrap(),
            vec![json!({ "created_at": { "order": "desc" } })]
        );
    }

    #[test]
    fn test_multiple_sort_fields_preserve_order() {
        let query = SearchQueryBuilder::new()
            .sort("priority", "desc")
            .sort("created_at", "asc")
            .build();
        let sort = query.sort.unwrap();
        assert_eq!(sort.len(), 2);
        assert_eq!(sort[0], json!({ "priority": { "order": "desc" } }));
        assert_eq!(sort[1], json!({ "created_at": { "order": "asc" } }));
    }

    #[test]
    fn test_no_sort_by_default() {
        let query = SearchQueryBuilder::new().build();
        assert!(query.sort.is_none());
    }

    // ── #1160: pagination edges ──────────────────────────────────────────

    #[test]
    fn test_default_pagination() {
        let query = SearchQuery::default();
        assert_eq!(query.from, 0);
        assert_eq!(query.size, 20);
    }

    #[test]
    fn test_zero_size_page() {
        let query = SearchQueryBuilder::new().size(0).build();
        assert_eq!(query.size, 0);
    }

    #[test]
    fn test_large_offset_and_size() {
        let query = SearchQueryBuilder::new().from(1_000_000).size(10_000).build();
        assert_eq!(query.from, 1_000_000);
        assert_eq!(query.size, 10_000);
    }

    #[test]
    fn test_pagination_survives_other_builder_calls() {
        let query = SearchQueryBuilder::new()
            .from(5)
            .match_query("title", "waste")
            .size(15)
            .sort("created_at", "desc")
            .build();
        assert_eq!(query.from, 5);
        assert_eq!(query.size, 15);
    }

    // ── #1160: highlight / aggregation / source ──────────────────────────

    #[test]
    fn test_highlight_json_shape() {
        let builder = SearchQueryBuilder::new().highlight(vec!["title".to_string(), "body".to_string()]);
        let body = builder.to_elasticsearch_json();
        assert_eq!(body["highlight"]["pre_tags"], json!(["<em>"]));
        assert_eq!(body["highlight"]["post_tags"], json!(["</em>"]));
        assert!(body["highlight"]["fields"]["title"].is_object());
        assert!(body["highlight"]["fields"]["body"].is_object());
    }

    #[test]
    fn test_aggregation_accumulates_multiple_names() {
        let builder = SearchQueryBuilder::new()
            .aggregation("by_status", json!({ "terms": { "field": "status" } }))
            .aggregation("by_category", json!({ "terms": { "field": "category" } }));
        let body = builder.to_elasticsearch_json();
        assert!(body["aggs"]["by_status"].is_object());
        assert!(body["aggs"]["by_category"].is_object());
    }

    #[test]
    fn test_source_field_filtering() {
        let query = SearchQueryBuilder::new()
            .source(vec!["id".to_string(), "title".to_string()])
            .build();
        assert_eq!(query.source, Some(vec!["id".to_string(), "title".to_string()]));
    }

    #[test]
    fn test_no_optional_fields_omitted_from_json_by_default() {
        let body = SearchQueryBuilder::new().to_elasticsearch_json();
        assert!(body.get("sort").is_none());
        assert!(body.get("highlight").is_none());
        assert!(body.get("aggs").is_none());
        assert!(body.get("_source").is_none());
    }

    // ── #1160: injection-safe parameter binding ───────────────────────────
    //
    // Every leaf value is placed via `serde_json::json!`/`Value`, never by
    // string-concatenating into a query body, so untrusted input can only
    // ever end up as a JSON string/number leaf — it cannot inject sibling
    // keys, escape into a different clause, or be reinterpreted as a nested
    // query object.

    #[test]
    fn test_match_query_treats_json_like_input_as_a_literal_string() {
        let malicious = r#"{"query": {"match_all": {}}}"#;
        let builder = SearchQueryBuilder::new().match_query("title", malicious);
        let body = builder.to_elasticsearch_json();
        // The payload stays exactly as supplied, as the *value* of "match",
        // not parsed into a sibling query clause.
        assert_eq!(body["query"]["match"]["title"], json!(malicious));
        assert_eq!(body["query"].as_object().unwrap().len(), 1);
    }

    #[test]
    fn test_field_name_with_special_characters_stays_a_single_key() {
        let field = r#"title"; DROP TABLE items; --"#;
        let builder = SearchQueryBuilder::new().match_query(field, "value");
        let body = builder.to_elasticsearch_json();
        assert_eq!(body["query"]["match"][field], json!("value"));
        assert_eq!(body["query"]["match"].as_object().unwrap().len(), 1);
    }

    #[test]
    fn test_term_query_preserves_non_string_value_types() {
        let builder = SearchQueryBuilder::new().term("count", json!(42));
        let body = builder.to_elasticsearch_json();
        assert_eq!(body["query"]["term"]["count"], json!(42));
        assert!(body["query"]["term"]["count"].is_number());
    }

    #[test]
    fn test_multi_match_query_with_quotes_and_braces_round_trips_verbatim() {
        let payload = r#"" OR 1=1 -- } { "malicious": true }"#;
        let builder = SearchQueryBuilder::new().multi_match(vec!["title".to_string()], payload);
        let body = builder.to_elasticsearch_json();
        assert_eq!(body["query"]["multi_match"]["query"], json!(payload));
    }

    #[test]
    fn test_wildcard_injection_attempt_stays_scoped_to_its_field() {
        let value = r#"*"}, "delete_all": {"match_all"#;
        let builder = SearchQueryBuilder::new().query(QueryType::Wildcard {
            field: "name".to_string(),
            value: value.to_string(),
        });
        let body = builder.to_elasticsearch_json();
        assert_eq!(body["query"], json!({ "wildcard": { "name": value } }));
        // No top-level key besides the intended "wildcard" was introduced.
        assert_eq!(body["query"].as_object().unwrap().keys().len(), 1);
    }

    #[test]
    fn test_bool_query_rejects_cross_clause_key_injection() {
        // A field name colliding with a reserved bool-clause name must not
        // let the term escape into `should`/`must_not` — it stays nested
        // under `must` as a term clause.
        let builder = SearchQueryBuilder::new().bool_query(
            vec![QueryType::Term {
                field: "should".to_string(),
                value: json!("hijack"),
            }],
            vec![],
            vec![],
        );
        let body = builder.to_elasticsearch_json();
        let must = body["query"]["bool"]["must"].as_array().unwrap();
        assert_eq!(must.len(), 1);
        assert_eq!(must[0], json!({ "term": { "should": "hijack" } }));
        assert!(body["query"]["bool"].get("should").is_none());
    }
}
