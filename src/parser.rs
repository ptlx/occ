use crate::types::{Node, Num, Parser, Token};

impl<'a> Parser<'a> {
    /// 構文木を作る処理
    pub fn parse(&mut self) -> Vec<Box<Node<'a>>> {
        self.program()
    }
    fn program(&mut self) -> Vec<Box<Node<'a>>> {
        let mut stmts: Vec<Box<Node>> = vec![];
        loop {
            let stmt = self.stmt();
            if let Some(node) = stmt {
                stmts.push(node);
            } else {
                break;
            }
        }
        stmts
    }
    fn stmt(&mut self) -> Option<Box<Node<'a>>> {
        let node;
        if self.token_iter.consume("return") {
            node = new_node(Token::Reserved("return"), self.expr(), None);
        } else if self.token_iter.consume("if") {
            // cond, then, elsが必要
            let cond: Option<Box<Node<'_>>>;
            let then: Option<Box<Node<'_>>>;
            // ない可能性があるため、Noneで初期化
            let mut els: Option<Box<Node<'_>>> = None;

            // if (cond) or if cond
            cond = self.expr();
            // then
            then = self.stmt();
            // もしelseが続いた場合、その後のstmtをelseの処理とみなす
            if self.token_iter.consume("else") {
                els = self.stmt();
            }
            node = new_node_if(cond, then, els);
        } else if self.token_iter.consume("for") {
            // for ([init], [cond]; [inc]) [stmt]
            let mut init: Option<Box<Node<'_>>> = None;
            let mut cond: Option<Box<Node<'_>>> = None;
            let mut inc: Option<Box<Node<'_>>> = None;
            let then: Option<Box<Node<'_>>>;
            self.token_iter.consume("(");
            // init
            if !self.token_iter.consume(";") {
                init = self.expr();
                self.token_iter.consume(";");
            }
            // cond
            if !self.token_iter.consume(";") {
                cond = self.expr();
                self.token_iter.consume(";");
            }
            if !self.token_iter.consume(")") {
                // inc
                inc = self.expr();
                self.token_iter.consume(")");
            }
            self.token_iter.consume(")");
            then = self.stmt();

            node = new_node_for(init, cond, inc, then);
        } else if self.token_iter.consume("while") {
            // while ([cond]) [then]
            let mut cond: Option<Box<Node<'_>>> = None;
            let then: Option<Box<Node<'_>>>;
            cond = self.expr();
            then = self.stmt();
            node = new_node_while(cond, then);
        } else {
            node = self.expr()
        }
        /*if !(self.token_iter.consume(";")) {
            return None;
        }*/
        self.token_iter.consume(";");
        node
    }
    fn assign(&mut self) -> Option<Box<Node<'a>>> {
        let mut node = self.equality();
        if self.token_iter.consume("=") {
            node = new_node(Token::Operand("="), node, self.assign())
        }
        node
    }

    fn primary(&mut self) -> Option<Box<Node<'a>>> {
        // 最初の数字をとっている想定
        let st = self.token_iter.next();
        if st.is_none() {
            return None;
        }
        let node: Option<Box<Node>>;
        // println!("{:?}",st);
        if *st.as_ref().unwrap() == Token::Operand("(") {
            node = self.expr();
            let token = self.token_iter.next();
            if token != Some(Token::Operand(")")) {
                panic!("')' expected, but found {:?}, self: {:?}", token, self)
            }
            node
        } else {
            match st {
                Some(Token::Num(n)) => new_node_num(n),
                Some(Token::LVar(var)) => {
                    self.vars.insert(var);
                    return new_node(Token::LVar(var), None, None);
                }
                _ => {
                    panic!("{:?}, {:?}", self, st);
                }
            }
        }
    }
    /// 式に相当する節
    fn expr(&mut self) -> Option<Box<Node<'a>>> {
        return self.assign();
    }

    fn equality(&mut self) -> Option<Box<Node<'a>>> {
        let node = self.relational();
        if self.token_iter.consume("==") {
            return new_node(Token::Operand("=="), node, self.relational());
        } else if self.token_iter.consume("!=") {
            return new_node(Token::Operand("!="), node, self.relational());
        } else {
            return node;
        }
    }

    fn add(&mut self) -> Option<Box<Node<'a>>> {
        let mut node = self.mul();
        'outer: loop {
            let operands = vec!["+", "-"].into_iter();
            for op in operands {
                if self.token_iter.consume(op) {
                    node = new_node(Token::Operand(op), node, self.mul());
                    continue 'outer;
                }
            }
            return node;
        }
    }

    fn relational(&mut self) -> Option<Box<Node<'a>>> {
        let mut node = self.add();
        'outer: loop {
            // そのまま構文木に入れる
            if self.token_iter.consume("<=") {
                node = new_node(Token::Operand("<="), node, self.add());
                continue 'outer;
            } else if self.token_iter.consume("<") {
                node = new_node(Token::Operand("<"), node, self.add());
                continue 'outer;
            }
            // 左右を反転させて対応した演算を指定し、構文木に入れる（例: "3>4"-> "4<3"）
            else if self.token_iter.consume("=>") {
                node = new_node(Token::Operand("<="), self.add(), node);
                continue 'outer;
            } else if self.token_iter.consume(">") {
                node = new_node(Token::Operand("<"), self.add(), node);
                continue 'outer;
            }
            return node;
        }
    }
    /// 乗法、除法に対応する節
    fn mul(&mut self) -> Option<Box<Node<'a>>> {
        // primary { * primary}
        let mut node = self.unary();
        loop {
            if self.token_iter.consume("*") {
                node = new_node(Token::Operand("*"), node, self.unary());
            } else if self.token_iter.consume("/") {
                node = new_node(Token::Operand("/"), node, self.unary());
            } else {
                return node;
            };
        }
    }

    fn unary(&mut self) -> Option<Box<Node<'a>>> {
        if self.token_iter.consume("+") {
            return self.primary();
        }
        if self.token_iter.consume("-") {
            return new_node(Token::Operand("-"), new_node_num(0), self.primary());
        }
        self.primary()
    }
}

/// 構文木を作るための補助的な関数
/// Some<Box<...>>でくるんで返す
fn new_node<'a>(
    kind: Token<'a>,
    lhs: Option<Box<Node<'a>>>,
    rhs: Option<Box<Node<'a>>>,
) -> Option<Box<Node<'a>>> {
    Some(Box::new(Node {
        kind,
        lhs,
        rhs,
        cond: None,
        then: None,
        els: None,
        init: None,
        inc: None,
    }))
}

/// 数字に対応した節を作る
fn new_node_num<'a>(val: Num) -> Option<Box<Node<'a>>> {
    let node = Node {
        kind: Token::Num(val),
        lhs: None,
        rhs: None,
        cond: None,
        then: None,
        els: None,
        init: None,
        inc: None,
    };
    Some(Box::new(node))
}

// ifに対応した節を作る
fn new_node_if<'a>(
    cond: Option<Box<Node<'a>>>,
    then: Option<Box<Node<'a>>>,
    els: Option<Box<Node<'a>>>,
) -> Option<Box<Node<'a>>> {
    let node = Node {
        kind: Token::Reserved("if"),
        lhs: None,
        rhs: None,
        cond,
        then,
        els,
        init: None,
        inc: None,
    };
    Some(Box::new(node))
}

// forに対応した節を作る
fn new_node_for<'a>(
    init: Option<Box<Node<'a>>>,
    cond: Option<Box<Node<'a>>>,
    inc: Option<Box<Node<'a>>>,
    then: Option<Box<Node<'a>>>,
) -> Option<Box<Node<'a>>> {
    let node = Node {
        kind: Token::Reserved("for"),
        lhs: None,
        rhs: None,
        cond,
        then,
        els: None,
        init,
        inc,
    };
    Some(Box::new(node))
}

// forに対応した節を作る
fn new_node_while<'a>(
    cond: Option<Box<Node<'a>>>,
    then: Option<Box<Node<'a>>>,
) -> Option<Box<Node<'a>>> {
    let node = Node {
        kind: Token::Reserved("while"),
        lhs: None,
        rhs: None,
        cond,
        then,
        els: None,
        init: None,
        inc: None,
    };
    Some(Box::new(node))
}

#[cfg(test)]
mod test {
    use std::collections::{HashMap, HashSet};

    use crate::types::{Parser, TokenIter, Variables};

    /*#[test]
    fn test_parser() {
        let mut iter = TokenIter { s: "1 < 2 + 3" };
        let mut vars = Variables {
            offsets: &mut HashMap::new()
        };
        let mut parser = Parser {
            token_iter: &mut iter,
            vars: &mut vars,
        };

        assert_eq!(
            *(parser.parse().unwrap()),
            Node {
                kind: Token::Operand("<"),
                lhs: new_node_num(1),
                rhs: Some(Box::new(Node {
                    kind: Token::Operand("+"),
                    lhs: new_node_num(2),
                    rhs: new_node_num(3)
                }))
            }
        )
    }*/
    #[test]
    fn test_set() {
        let mut set: HashSet<i32> = HashSet::new();
        set.insert(1);
        set.insert(2);
        assert_eq!(set.len(), 2)
    }

    #[test]
    fn test_stmt() {
        let mut iter = TokenIter { s: "var = 1; var;" };
        let mut vars = Variables {
            offsets: &mut HashMap::new(),
        };
        let mut parser = Parser {
            token_iter: &mut iter,
            vars: &mut vars,
        };
        println!("{:?}", parser.parse());
        println!("{:?}", parser);
    }

    #[test]
    fn debug_if() {
        let mut iter = TokenIter {
            s: "if 3 == 1 a=1;else a=0;",
        };
        let mut vars = Variables {
            offsets: &mut HashMap::new(),
        };
        let mut parser = Parser {
            token_iter: &mut iter,
            vars: &mut vars,
        };
        println!("{:?}", parser.parse());
        println!("{:?}", parser);
    }

    #[test]
    fn debug_for() {
        let mut iter = TokenIter {
            s: "for(i=0;;) return 0;",
        };
        let mut vars = Variables {
            offsets: &mut HashMap::new(),
        };
        let mut parser = Parser {
            token_iter: &mut iter,
            vars: &mut vars,
        };
        println!("{:?}", parser.parse());
        println!("{:?}", parser);
    }
}
