#![cfg(test)]

use super::*;
use serde_json::json;

#[test] fn test_openapi_doc_serialization() {
    let doc = Document::new(
        "Sample API", "0.1.9", [
            Server::at("http://api.example.com/v1")
                .description("Optional server description, e.g. Main (production) server"),
            Server::at("http://staging-api.example.com")
                .description("Optional server description, e.g. Internal staging server for testing")
        ]
    )
    .description("Optional multiline or single-line description in [CommonMark](http://commonmark.org/help/) or HTML.")
    .path("/users", Operations::new()
        .get(
            Operation::with(
                Responses::new(200, Response::of("A JSON array of user names")
                    .content("application/json", Schema::array().items(Schema::string()))
                )
            )
            .summary("Returns a list of users.")
            .description("Optional extended description in CommonMark or HTML.")
        )
    );

    assert_eq!(serde_json::to_value(doc).unwrap(), json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Sample API",
            "description": "Optional multiline or single-line description in [CommonMark](http://commonmark.org/help/) or HTML.",
            "version": "0.1.9"
        },
        "servers": [
            {
                "url": "http://api.example.com/v1",
                "description": "Optional server description, e.g. Main (production) server"
            },
            {
                "url": "http://staging-api.example.com",
                "description": "Optional server description, e.g. Internal staging server for testing"
            }
        ],
        "paths": {
            "/users": {
                "get": {
                    "summary": "Returns a list of users.",
                    "description": "Optional extended description in CommonMark or HTML.",
                    "responses": {
                        "200": {
                            "description": "A JSON array of user names",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {
                                            "type": "string"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }));
}
