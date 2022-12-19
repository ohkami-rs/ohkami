use crate::{
    components::method::Method,
    // === actual Handler ===
    // server::Handler,
    // ======================
};

// === mock for test ===
pub(self) type Handler = usize;
// =====================

mod pattern;

mod node;
use node::Node;


#[derive(PartialEq, Debug)]
#[allow(non_snake_case)]
pub(crate) struct Router<'p> {
    GET:    Node<'p>,
    POST:   Node<'p>,
    PATCH:  Node<'p>,
    DELETE: Node<'p>,
}
impl<'p> Router<'p> {
    pub fn new() -> Self {
        Self {
            GET:    Node::new(""),
            POST:   Node::new(""),
            PATCH:  Node::new(""),
            DELETE: Node::new(""),
        }
    }
    pub fn register(&mut self,
        method:       Method,
        path_pattern: &'static str,
        handler:      Handler,
    ) -> std::result::Result<(), String> {
        let err_msg = format!("path pattern `{path_pattern}` is resistred duplicatedly");

        let mut path = path_pattern.split('/');
        { path.next(); }

        let tree = match method {
            Method::GET    => &mut self.GET,
            Method::POST   => &mut self.POST,
            Method::PATCH  => &mut self.PATCH,
            Method::DELETE => &mut self.DELETE,
        };
        
        tree.register(path, handler, err_msg)
    }
    pub fn search(&self,
        method:       Method,
        request_path: &'p str,
    ) -> Option<&Handler> {
        let mut path = request_path.split('/');
        { path.next(); }

        let tree = match method {
            Method::GET    => &self.GET,
            Method::POST   => &self.POST,
            Method::PATCH  => &self.PATCH,
            Method::DELETE => &self.DELETE,
        };

        tree.search(path)
    }
}





// to run tests,
// - commentout `Server::Handler` in `use::crate::{}`
// - uncommentout `type Handler = usize`
#[cfg(test)]
mod test_resister {
    #![allow(unused)]
    use crate::{test_system::Method::*, context::Context, response::Response, result::Result};
    use super::{Router, Node, pattern::Pattern};

    #[test]
    fn register_one_1() {
        let mock_handler = 100;

        let mut router = Router::new();
        router.register(GET, "/", mock_handler);

        assert_eq!(
            router,
            Router {
                GET: Node {
                    pattern:  Pattern::Str(""),
                    handler:  None,
                    children: vec![
                        Node {
                            pattern:  Pattern::Str(""),
                            handler:  Some(mock_handler),
                            children: vec![]
                        }
                    ],
                },
                POST:   Node::new(""),
                PATCH:  Node::new(""),
                DELETE: Node::new(""),
            }
        )
    }
    #[test]
    fn register_one_2() {
        let mock_handler = 100;

        let mut router = Router::new();
        router.register(GET, "/api", mock_handler);

        assert_eq!(
            router,
            Router {
                GET: Node {
                    pattern:  Pattern::Str(""),
                    handler:  None,
                    children: vec![
                        Node {
                            pattern:  Pattern::Str("api"),
                            handler:  Some(mock_handler),
                            children: vec![]
                        }
                    ],
                },
                POST:   Node::new(""),
                PATCH:  Node::new(""),
                DELETE: Node::new(""),
            }
        )
    }

    #[test]
    fn register_two_pararel_1() {
        let (mock_handler_1, mock_handler_2) = (100, 200);

        let mut router = Router::new();
        router.register(GET, "/api", mock_handler_1);
        router.register(POST, "/api", mock_handler_2);

        assert_eq!(
            router,
            Router {
                GET: Node {
                    pattern:  Pattern::Str(""),
                    handler:  None,
                    children: vec![
                        Node {
                            pattern:  Pattern::Str("api"),
                            handler:  Some(mock_handler_1),
                            children: vec![]
                        }
                    ],
                },
                POST: Node {
                    pattern:  Pattern::Str(""),
                    handler:  None,
                    children: vec![
                        Node {
                            pattern:  Pattern::Str("api"),
                            handler:  Some(mock_handler_2),
                            children: vec![]
                        }
                    ],
                },
                PATCH:  Node::new(""),
                DELETE: Node::new(""),
            }
        )
    }
    #[test]
    fn register_two_pararel_2() {
        let (mock_handler_1, mock_handler_2) = (100, 200);

        let mut router = Router::new();
        router.register(GET, "/api", mock_handler_1);
        router.register(POST, "/api_v2", mock_handler_2);

        assert_eq!(
            router,
            Router {
                GET: Node {
                    pattern:  Pattern::Str(""),
                    handler:  None,
                    children: vec![
                        Node {
                            pattern:  Pattern::Str("api"),
                            handler:  Some(mock_handler_1),
                            children: vec![]
                        }
                    ],
                },
                POST: Node {
                    pattern:  Pattern::Str(""),
                    handler:  None,
                    children: vec![
                        Node {
                            pattern:  Pattern::Str("api_v2"),
                            handler:  Some(mock_handler_2),
                            children: vec![]
                        }
                    ],
                },
                PATCH:  Node::new(""),
                DELETE: Node::new(""),
            }
        )
    }
    #[test]
    fn register_two_pararel_3() {
        let (mock_handler_1, mock_handler_2) = (100, 200);

        let mut router = Router::new();
        router.register(GET, "/api", mock_handler_1);
        router.register(GET, "/api_v2", mock_handler_2);

        assert_eq!(
            router,
            Router {
                GET: Node {
                    pattern:  Pattern::Str(""),
                    handler:  None,
                    children: vec![
                        Node {
                            pattern:  Pattern::Str("api"),
                            handler:  Some(mock_handler_1),
                            children: vec![]
                        },
                        Node {
                            pattern:  Pattern::Str("api_v2"),
                            handler:  Some(mock_handler_2),
                            children: vec![]
                        },
                    ],
                },
                POST:   Node::new(""),
                PATCH:  Node::new(""),
                DELETE: Node::new(""),
            }
        )
    }
    
    #[test]
    fn register_two_nested_1() {
        let (mock_handler_1, mock_handler_2) = (100, 200);

        let mut router = Router::new();
        router.register(GET, "/api", mock_handler_1);
        router.register(GET, "/api/users", mock_handler_2);

        assert_eq!(
            router,
            Router {
                GET: Node {
                    pattern:  Pattern::Str(""),
                    handler:  None,
                    children: vec![
                        Node {
                            pattern:  Pattern::Str("api"),
                            handler:  Some(mock_handler_1),
                            children: vec![
                                Node {
                                    pattern:  Pattern::Str("users"),
                                    handler:  Some(mock_handler_2),
                                    children: vec![]
                                }
                            ]
                        }
                    ],
                },
                POST:   Node::new(""),
                PATCH:  Node::new(""),
                DELETE: Node::new(""),
            }
        )
    }

    #[test]
    fn register_three_1() {
        let (mock_handler_1, mock_handler_2, mock_handler_3) = (100, 200, 300);

        let mut router = Router::new();
        router.register(GET, "/api", mock_handler_1);
        router.register(GET, "/api/users", mock_handler_2);
        router.register(GET, "/api/articles", mock_handler_3);

        assert_eq!(
            router,
            Router {
                GET: Node {
                    pattern:  Pattern::Str(""),
                    handler:  None,
                    children: vec![
                        Node {
                            pattern:  Pattern::Str("api"),
                            handler:  Some(mock_handler_1),
                            children: vec![
                                Node {
                                    pattern:  Pattern::Str("users"),
                                    handler:  Some(mock_handler_2),
                                    children: vec![]
                                },
                                Node {
                                    pattern:  Pattern::Str("articles"),
                                    handler:  Some(mock_handler_3),
                                    children: vec![]
                                },
                            ]
                        }
                    ],
                },
                POST:   Node::new(""),
                PATCH:  Node::new(""),
                DELETE: Node::new(""),
            }
        )
    }
    #[test]
    fn register_three_2() {
        let (mock_handler_1, mock_handler_2, mock_handler_3) = (100, 200, 300);

        let mut router = Router::new();
        router.register(GET, "/api", mock_handler_1);
        router.register(GET, "/api/users", mock_handler_2);
        router.register(POST, "/api", mock_handler_3);

        assert_eq!(
            router,
            Router {
                GET: Node {
                    pattern:  Pattern::Str(""),
                    handler:  None,
                    children: vec![
                        Node {
                            pattern:  Pattern::Str("api"),
                            handler:  Some(mock_handler_1),
                            children: vec![
                                Node {
                                    pattern:  Pattern::Str("users"),
                                    handler:  Some(mock_handler_2),
                                    children: vec![]
                                },
                            ]
                        }
                    ],
                },
                POST: Node {
                    pattern:  Pattern::Str(""),
                    handler:  None,
                    children: vec![
                        Node {
                            pattern:  Pattern::Str("api"),
                            handler:  Some(mock_handler_3),
                            children: vec![]
                        }
                    ],
                },
                PATCH:  Node::new(""),
                DELETE: Node::new(""),
            }
        )
    }
}
