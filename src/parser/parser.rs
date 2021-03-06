use regex::Regex;
use lazy_static::lazy_static;

use crate::tokenizer::Token;
use super::{ParseTreeNode, ExpressionType};
use crate::cli::Error;
use super::error::{expected_got_error, unexpected_eof_error};

static TYPES: &[&str] = &["i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "void"];
static KEYWORDS: &[&str] = &["loop", "while", "if", "break", "continue", "else", "do", "as"];
static MAX_EXPRESSION: usize = 17;

lazy_static!
{
    static ref IDENTIFIER_REGEX: Regex = Regex::new(r"\A[a-zA-Z|_][a-zA-Z0-9|_]*").unwrap();
    static ref INTEGER_REGEX: Regex = Regex::new(r"\A[0-9]+").unwrap();
}

/// Token Stream
#[derive(Debug, Clone)]
pub struct Stream
{
    tokens: Vec<Token>,
    index: usize
}

impl Stream
{
    /// Generate a new token stream object
    pub fn new(tokens: Vec<Token>) -> Self
    {
        Self
        {
            tokens,
            index: 0
        }
    }

    /// Peek at the next token
    pub fn peek(&self) -> Option<Token>
    {
        if self.index + 1 < self.tokens.len()
        {
            Some(self.tokens[self.index + 1].clone())
        }
        else
        {
            None
        }
    }

    /// Get the current token
    pub fn current(&self) -> Option<Token>
    {
        if self.index < self.tokens.len()
        {
            Some(self.tokens[self.index].clone())
        }
        else
        {
            None
        }
    }

    /// Consume the curent token
    pub fn consume(&mut self) -> bool
    {
        self.index += 1;
        self.index < self.tokens.len()
    }

    /// Check the next token
    pub fn check_next(&self, data: String) -> bool
    {
        (self.index + 1 < self.tokens.len()) && (self.peek().unwrap().data == data)
    }

    /// Check the current token
    pub fn check_current(&self, data: String) -> bool
    {
        (self.index < self.tokens.len()) && (self.current().unwrap().data == data)
    }

    /// Expect a token at an offset from the current token
    fn expect_at(&self, data: String, index: usize) -> Result<(), Error>
    {
        if index < self.tokens.len()
        {
            let got = self.tokens[index].data.clone();
            if data == got
            {
                Ok(())
            }
            else
            {
                expected_got_error(&format!("'{}'", data), &self.tokens[index])
            }
        }
        else
        {
            unexpected_eof_error(&format!("'{}'", data), &self.tokens[index - 1])
        }
    }

    /// Expect a token at the current position
    pub fn expect(&self, data: String) -> Result<(), Error>
    {
        self.expect_at(data, self.index)
    }

    /// Expect and consume a token
    pub fn expect_and_consume(&mut self, data: String) -> Result<(), Error>
    {
        match self.expect_at(data, self.index)
        {
            Ok(()) => {self.consume(); Ok(())},
            Err(e) => Err(e)
        }
    }
    
    /// Expect the next token
    pub fn expect_next(&self, data: String) -> Result<(), Error>
    {
        self.expect_at(data, self.index + 1)
    }

    /// Expect that the EOF hasn't been reached
    pub fn expect_current_exists(&self, s: &str) -> Result<(), Error>
    {
        if self.current().is_none()
        {
            unexpected_eof_error(s, &self.tokens[self.index - 1])
        }
        else
        {
            Ok(())
        }
    }

    /// Accept a stream
    pub fn accept_stream(&mut self, result: Result<(Stream, ParseTreeNode), Error>) -> Result<ParseTreeNode, Error>
    {
        let val = result?;
        self.tokens = val.0.tokens;
        self.index = val.0.index;

        Ok(val.1)
    }
}

/// Convert a parse tree to be on the left hand side
pub fn convert_to_left(node: ParseTreeNode)  -> Result<ParseTreeNode, Error>
{
    let mut tree = node.clone();
    
    tree = match &tree
    {
        ParseTreeNode::Expression(expr_type, children) =>
        {
            if expr_type == &ExpressionType::Dereference
            {
                ParseTreeNode::Expression(ExpressionType::DereferenceLeft, children.clone())
            }
            else
            {
                tree
            }
        },
        _ => tree
    };

    tree = match tree
    {
        ParseTreeNode::Library(c) =>
        {
            let mut new_children = vec![];

            for child in c
            {
                new_children.push(convert_to_left(child)?);
            }

            ParseTreeNode::Library(new_children)
        },
        ParseTreeNode::Function(c) =>
        {
            let mut new_children = vec![];

            for child in c
            {
                new_children.push(convert_to_left(child)?);
            }

            ParseTreeNode::Function(new_children)
        },
        ParseTreeNode::Arguments(c) =>
        {
            let mut new_children = vec![];

            for child in c
            {
                new_children.push(convert_to_left(child)?);
            }

            ParseTreeNode::Arguments(new_children)
        },
        ParseTreeNode::Argument(c) =>
        {
            let mut new_children = vec![];

            for child in c
            {
                new_children.push(convert_to_left(child)?);
            }

            ParseTreeNode::Argument(new_children)
        },
        ParseTreeNode::Type(c) =>
        {
            let mut new_children = vec![];

            for child in c
            {
                new_children.push(convert_to_left(child)?);
            }

            ParseTreeNode::Type(new_children)
        },
        ParseTreeNode::Statement(c) =>
        {
            let mut new_children = vec![];

            for child in c
            {
                new_children.push(convert_to_left(child)?);
            }

            ParseTreeNode::Statement(new_children)
        },
        ParseTreeNode::Statements(c) =>
        {
            let mut new_children = vec![];

            for child in c
            {
                new_children.push(convert_to_left(child)?);
            }

            ParseTreeNode::Statements(new_children)
        },
        ParseTreeNode::Assignments(c) =>
        {
            let mut new_children = vec![];

            for child in c
            {
                new_children.push(convert_to_left(child)?);
            }

            ParseTreeNode::Assignments(new_children)
        },
        ParseTreeNode::Assignment(c) =>
        {
            let mut new_children = vec![];

            for child in c
            {
                new_children.push(convert_to_left(child)?);
            }

            ParseTreeNode::Assignment(new_children)
        },
        ParseTreeNode::Expression(t, c) =>
        {
            let mut new_children = vec![];

            for child in c
            {
                new_children.push(convert_to_left(child)?);
            }

            ParseTreeNode::Expression(t, new_children)
        },
        ParseTreeNode::AssignmentStatement(c) =>
        {
            let mut new_children = vec![];

            for child in c
            {
                new_children.push(convert_to_left(child)?);
            }

            ParseTreeNode::AssignmentStatement(new_children)
        },

        ParseTreeNode::IfStatement(c) =>
        {
            let mut new_children = vec![];

            for child in c
            {
                new_children.push(convert_to_left(child)?);
            }

            ParseTreeNode::IfStatement(new_children)
        },
        ParseTreeNode::ReturnStatement(c) =>
        {
            let mut new_children = vec![];

            for child in c
            {
                new_children.push(convert_to_left(child)?);
            }

            ParseTreeNode::ReturnStatement(new_children)
        },
        ParseTreeNode::WhileLoop(c) =>
        {
            let mut new_children = vec![];

            for child in c
            {
                new_children.push(convert_to_left(child)?);
            }

            ParseTreeNode::WhileLoop(new_children)
        },
        ParseTreeNode::DoWhileLoop(c) =>
        {
            let mut new_children = vec![];

            for child in c
            {
                new_children.push(convert_to_left(child)?);
            }

            ParseTreeNode::DoWhileLoop(new_children)
        },
        ParseTreeNode::Loop(c) =>
        {
            let mut new_children = vec![];

            for child in c
            {
                new_children.push(convert_to_left(child)?);
            }

            ParseTreeNode::Loop(new_children)
        },
        _ => tree
    };

    Ok(tree)
}

/// Get the parse tree for a translation unit
pub fn parse(tokens: Vec<Token>) -> Result<ParseTreeNode, Error>
{
    Ok(parse_library(&Stream::new(tokens))?.1)
}

/// Parse out a raw type
/// (for example i8, but not i8*)
fn parse_raw_type(orig_stream: &Stream) -> Result<(Stream, ParseTreeNode), Error>
{
    let mut stream = orig_stream.clone();

    // Make sure the stream isn't exhausted
    stream.expect_current_exists("raw type")?;

    let val = stream.current().unwrap();

    if TYPES.contains(&val.data.as_str())
    {
        stream.consume();
        Ok((stream, ParseTreeNode::RawType(val.clone())))
    }
    else
    {
        expected_got_error("raw type", &val)
    }
}

/// Parse out an identifier
fn parse_identifier(orig_stream: &Stream)  -> Result<(Stream, ParseTreeNode), Error>
{
    let mut stream = orig_stream.clone();

    // Make sure the stream isn't exhausted
    stream.expect_current_exists("identifier")?;

    let val = stream.current().unwrap();

    
        // A type instead
    if  TYPES.contains(&val.data.as_str()) ||  

        // A keyword
        KEYWORDS.contains(&val.data.as_str()) ||
        // Doesn't match the identifier regex
        !IDENTIFIER_REGEX.is_match(val.data.as_str())
    {
        return expected_got_error("identifier",&val);
    }

    stream.consume();
    Ok((stream, ParseTreeNode::Identifier(val.clone())))
}

/// Parse out an integer
fn parse_integer(orig_stream: &Stream)  -> Result<(Stream, ParseTreeNode), Error>
{
    let mut stream = orig_stream.clone();

    // Make sure the stream isn't exhausted
    stream.expect_current_exists("integer")?;

    let val = stream.current().unwrap();

    if !INTEGER_REGEX.is_match(val.data.as_str())
    {
        return expected_got_error("integer",&val);
    }

    stream.consume();
    Ok((stream, ParseTreeNode::IntegerLiteral(val.clone())))
}

/// Parse out a single token
fn parse_token(orig_stream: &Stream, what: &str) -> Result<(Stream, ParseTreeNode), Error>
{
    let mut stream = orig_stream.clone();

    // Make sure the stream isn't exhausted
    stream.expect_current_exists(what)?;

    let val = stream.current().unwrap();
    stream.consume();

    Ok((stream, ParseTreeNode::RawToken(val)))
}

/// Parse out a type
/// Either just a raw type or a raw type folloed by multiple '*'s
/// Parse out an identifier
fn parse_type(orig_stream: &Stream)  -> Result<(Stream, ParseTreeNode), Error>
{
    let mut stream = orig_stream.clone();

    // Make sure the stream isn't exhausted
    stream.expect_current_exists("type")?;

    // First is getting the raw type
    let raw_type = stream.accept_stream(parse_raw_type(&stream))?;
    let mut items = vec![raw_type];

    while stream.check_current(String::from("*"))
    {
        items.push(ParseTreeNode::RawToken(stream.current().unwrap()));
        stream.consume();
    }

    Ok((stream, ParseTreeNode::Type(items)))
}

/// Recursive Parsing of expressions
fn recursive_expression(orig_stream: &Stream, depth: usize) -> Result<(Stream, ParseTreeNode), Error>
{
    let mut stream = orig_stream.clone();

    match depth
    {
        // Number, Identifier, (expr)
        0 => 
        {
            if stream.check_current(String::from("("))
            {
                // Open Paren
                stream.expect_and_consume(String::from("("))?;

                let val = stream.accept_stream(recursive_expression(&stream, MAX_EXPRESSION))?;

                // Close Paren
                stream.expect_and_consume(String::from(")"))?;

                Ok((stream, val))
            }
            else if let Ok(val) = parse_integer(&stream)
            {
                Ok(val)
            }
            else if let Ok(val) = parse_identifier(&stream)
            {
                Ok(val)
            }
            else
            {
                expected_got_error("expression", &stream.current().unwrap())
            }
        },
        // Array Access or Function Call
        1 =>
        {
            // Start with a previous expression
            let mut current = stream.accept_stream(recursive_expression(&stream, depth - 1))?;

            // Loop because this operation is left associative
            while stream.check_current(String::from("[")) || stream.check_current(String::from("("))
            {
                // Array access
                if stream.check_current(String::from("["))
                {
                    // Open bracket
                    stream.expect_and_consume(String::from("["))?;

                    // Get the internal expression
                    let expr = stream.accept_stream(parse_expression(&stream))?;

                    // Close bracket
                    stream.expect_and_consume(String::from("]"))?;

                    current = ParseTreeNode::Expression(ExpressionType::ArrayAccess, vec![current, expr]);
                }
                // Function Call
                else
                {
                    // Open bracket
                    stream.expect_and_consume(String::from("("))?;

                    // Array starts with the current expression
                    let mut items = vec![current];

                    while !stream.check_current(String::from(")"))
                    {
                        items.push(stream.accept_stream(recursive_expression(&stream, MAX_EXPRESSION - 1))?);

                        if stream.check_current(String::from(","))
                        {
                            stream.expect_and_consume(String::from(","))?;
                        }
                        else
                        {
                            break;
                        }
                    }

                    // Close bracket
                    stream.expect_and_consume(String::from(")"))?;

                    current = ParseTreeNode::Expression(ExpressionType::FunctionCall, items);
                }
            }

            Ok((stream, current))
        },
        // Post Increment and Post Decrement
        2 =>
        {
            // Start with a previous expression
            let mut current = stream.accept_stream(recursive_expression(&stream, depth - 1))?;

            // Loop because this operation is left associative
            while stream.check_current(String::from("++")) || stream.check_current(String::from("--"))
            {
                // Post Increment
                if stream.check_current(String::from("++"))
                {
                    // Opperation
                    stream.expect_and_consume(String::from("++"))?;
                    
                    current = ParseTreeNode::Expression(ExpressionType::PostIncrement, vec![current]);
                }
                // Post Decrement
                else
                {
                    // Opperation
                    stream.expect_and_consume(String::from("--"))?;
                    
                    current = ParseTreeNode::Expression(ExpressionType::PostDecrement, vec![current]);
                }
            }

            Ok((stream, current))
        },
        // Prefix Operators
        3 =>
        {
            let op = match stream.current().unwrap().data.as_str()
            {
                "++" => Some(ExpressionType::PreIncrement),
                "--" => Some(ExpressionType::PreDecrement),
                "+" => Some(ExpressionType::UnaryPlus),
                "-" => Some(ExpressionType::UnaryMinus),
                "!" => Some(ExpressionType::LogicalNot),
                "~" => Some(ExpressionType::BitwiseNot),
                "*" => Some(ExpressionType::Dereference),
                "&" => Some(ExpressionType::Reference),
                _ => None
            };

            if op.is_none()
            {
                recursive_expression(&stream, depth - 1)
            }
            else
            {
                stream.consume();

                let post = stream.accept_stream(recursive_expression(&stream, depth))?;
                Ok((stream, ParseTreeNode::Expression(op.unwrap(), vec![post])))
            }
        },
        // Binary Operators
        4..=13 | 15 | 17 =>
        {
            let mut prev = stream.accept_stream(recursive_expression(&stream, depth - 1))?;
            
            // Extract the operator
            let op = 
            if let Some(current) = stream.current()
            {
                match depth
                {
                    // Multiplicative Operations
                    4 =>
                    {
                        match current.data.as_str()
                        {
                            "*" => Some(ExpressionType::Multiply),
                            "/" => Some(ExpressionType::Divide),
                            "%" => Some(ExpressionType::Modulus),
                            _ => None
                        }
                    },
                    // Additive Operations
                    5 =>
                    {
                        match current.data.as_str()
                        {
                            "+" => Some(ExpressionType::Add),
                            "-" => Some(ExpressionType::Subtract),
                            _ => None
                        }
                    },
                    // Shift Operations
                    6 =>
                    {
                        match current.data.as_str()
                        {
                            "<<" => Some(ExpressionType::ShiftLeft),
                            ">>" => Some(ExpressionType::ShiftRight),
                            _ => None
                        }
                    },
                    // Comparison Operations
                    7 =>
                    {
                        match current.data.as_str()
                        {
                            "<" => Some(ExpressionType::LessThan),
                            "<=" => Some(ExpressionType::LessThanOrEqual),
                            ">" => Some(ExpressionType::GreaterThan),
                            ">=" => Some(ExpressionType::GreaterThanOrEqual),
                            _ => None
                        }
                    },
                    // Equality Operations
                    8 =>
                    {
                        match current.data.as_str()
                        {
                            "==" => Some(ExpressionType::Equal),
                            "!=" => Some(ExpressionType::NotEqual),
                            _ => None
                        }
                    },
                    // Bitwise And
                    9 =>
                    {
                        match current.data.as_str()
                        {
                            "&" => Some(ExpressionType::BitwiseAnd),
                            _ => None
                        }
                    },
                    // Bitwise Xor
                    10 =>
                    {
                        match current.data.as_str()
                        {
                            "^" => Some(ExpressionType::BitwiseXor),
                            _ => None
                        }
                    },
                    // Bitwise Or
                    11 =>
                    {
                        match current.data.as_str()
                        {
                            "|" => Some(ExpressionType::BitwiseOr),
                            _ => None
                        }
                    },
                    // Logical And
                    12 =>
                    {
                        match current.data.as_str()
                        {
                            "&&" => Some(ExpressionType::LogicalAnd),
                            _ => None
                        }
                    },
                    // Logical Or
                    13 =>
                    {
                        match current.data.as_str()
                        {
                            "||" => Some(ExpressionType::LogicalOr),
                            _ => None
                        }
                    },
                    // Assignment Operators
                    15 =>
                    {
                        match current.data.as_str()
                        {
                            "=" => Some(ExpressionType::Assignment),
                            "+=" => Some(ExpressionType::AddAssign),
                            "-=" => Some(ExpressionType::SubtractAssign),
                            "*=" => Some(ExpressionType::MultiplyAssign),
                            "/=" => Some(ExpressionType::DivideAssign),
                            "%=" => Some(ExpressionType::ModulusAssign),
                            "<<=" => Some(ExpressionType::ShiftLeftAssign),
                            ">>=" => Some(ExpressionType::ShiftRightAssign),
                            "&=" => Some(ExpressionType::BitwiseAnd),
                            "^=" => Some(ExpressionType::BitwiseXor),
                            "|=" => Some(ExpressionType::BitwiseOr),
                            _ => None
                        }
                    },
                    // Comma
                    17 =>
                    {
                        match current.data.as_str()
                        {
                            "," => Some(ExpressionType::Comma),
                            _ => None
                        }
                    },
                    _ => None
                }
                
            }
            // If we have reached the EOF, there can't be an operator
            else
            {
                None
            };

            if op.is_none()
            {
                Ok((stream, prev))
            }
            else
            {
                stream.consume();

                // We know the expression is an assignment
                if depth == 15
                {
                    prev = convert_to_left(prev)?;
                }

                let post = stream.accept_stream(recursive_expression(&stream, depth))?;
                Ok((stream, ParseTreeNode::Expression(op.unwrap(), vec![prev, post])))
            }
        },
        // Ternary Operator
        14 =>
        {
            let prev = stream.accept_stream(recursive_expression(&stream, depth - 1))?;

            if stream.check_current(String::from("?"))
            {
                stream.expect_and_consume(String::from("?"))?;
                let inner = stream.accept_stream(parse_expression(&stream))?;
                stream.expect_and_consume(String::from(":"))?;
                let last = stream.accept_stream(recursive_expression(&stream, depth))?;

                Ok((stream, ParseTreeNode::Expression(ExpressionType::Ternary, vec![prev, inner, last])))
            }
            else
            {
                Ok((stream, prev))
            }
        },
        // Cast
        16 =>
        {
            let prev = stream.accept_stream(recursive_expression(&stream, depth - 1))?;

            if stream.check_current(String::from("as"))
            {
                stream.expect_and_consume(String::from("as"))?;
                let datatype = stream.accept_stream(parse_type(&stream))?;

                Ok((stream, ParseTreeNode::Expression(ExpressionType::Cast, vec![prev, datatype])))
            }
            else
            {
                Ok((stream, prev))
            }
        },
        default => panic!("Unexpected depth value of {}", default)
    }
}

/// Parse out an expression
fn parse_expression(orig_stream: &Stream)  -> Result<(Stream, ParseTreeNode), Error>
{
    let stream = orig_stream.clone();

    // Make sure the stream isn't exhausted
    stream.expect_current_exists("expression")?;

    // Simplest expression is just a number
    recursive_expression(&stream, MAX_EXPRESSION)
}

/// Parse out an expression without comma expressions
fn parse_expression_no_comma(orig_stream: &Stream)  -> Result<(Stream, ParseTreeNode), Error>
{
    let stream = orig_stream.clone();

    // Make sure the stream isn't exhausted
    stream.expect_current_exists("expression")?;

    // Simplest expression is just a number
    recursive_expression(&stream, MAX_EXPRESSION - 1)
}

/// Parse out an assignment
fn parse_assignment(orig_stream: &Stream)  -> Result<(Stream, ParseTreeNode), Error>
{
    let mut stream = orig_stream.clone();

    // Make sure the stream isn't exhausted    
    stream.expect_current_exists("assignment")?;

    // First is an identifier
    let identifier = stream.accept_stream(parse_identifier(&stream))?;
    let mut items = vec![identifier];

    // Next is an equals sign
    stream.expect_and_consume(String::from("="))?;

    // Finally, an expression
    let expr = stream.accept_stream(parse_expression_no_comma(&stream))?;
    items.push(expr);

    Ok((stream, ParseTreeNode::Assignment(items)))
}

/// Parse out assignments
fn parse_assignments(orig_stream: &Stream)  -> Result<(Stream, ParseTreeNode), Error>
{
    let mut stream = orig_stream.clone();

    // Purposeful removal, it doesn't make sense to say we are looking for arguments here, the error
    // message could get somewhat confusing
    //stream.expect_current_exists("arguments")?;

    // First is getting the return type
    let arg = stream.accept_stream(parse_assignment(&stream))?;
    let mut items = vec![arg];

    while stream.check_current(String::from(","))
    {
        stream.consume();
        items.push(stream.accept_stream(parse_assignment(&stream))?);
    }

    Ok((stream, ParseTreeNode::Assignments(items)))
}

/// Parse out an if statement
fn parse_if_statement(orig_stream: &Stream)  -> Result<(Stream, ParseTreeNode), Error>
{
    let mut stream = orig_stream.clone();

    // Make sure the stream isn't exhausted
    stream.expect_current_exists("if statement")?;

    // Must start with an if keyword
    stream.expect_and_consume(String::from("if"))?;

    // Get the condition
    let cond = stream.accept_stream(parse_expression(&stream))?;
    let mut items = vec![cond];

    // Get the main body
    let body = stream.accept_stream(parse_statement(&stream))?;
    items.push(body);

    // Check if there is an else clause
    if stream.check_current(String::from("else"))
    {
        stream.expect_and_consume(String::from("else"))?;

        let clause = stream.accept_stream(parse_statement(&stream))?;
        items.push(clause);
    }
    else
    {
        items.push(ParseTreeNode::Empty);
    }

    Ok((stream, ParseTreeNode::IfStatement(items)))
}

/// Parse out a while loop
fn parse_while_loop(orig_stream: &Stream)  -> Result<(Stream, ParseTreeNode), Error>
{
    let mut stream = orig_stream.clone();

    // Make sure the stream isn't exhausted
    stream.expect_current_exists("while loop")?;

    // Must start with a while keyword
    stream.expect_and_consume(String::from("while"))?;

    let cond = stream.accept_stream(parse_expression(&stream))?;
    
    let statement = stream.accept_stream(parse_statement(&stream))?;

    
    Ok((stream, ParseTreeNode::WhileLoop(vec![cond, statement])))
}

/// Parse out a do while loop
fn parse_do_while_loop(orig_stream: &Stream)  -> Result<(Stream, ParseTreeNode), Error>
{
    let mut stream = orig_stream.clone();

    // Make sure the stream isn't exhausted
    stream.expect_current_exists("do while loop")?;

    // Must start with a do keyword
    stream.expect_and_consume(String::from("do"))?;

    let statement = stream.accept_stream(parse_statement(&stream))?;

    // Must be followed by a while keyword
    stream.expect_and_consume(String::from("while"))?;

    let cond = stream.accept_stream(parse_expression(&stream))?;
    
    Ok((stream, ParseTreeNode::DoWhileLoop(vec![cond, statement])))
}

/// Parse out a loop
fn parse_loop(orig_stream: &Stream)  -> Result<(Stream, ParseTreeNode), Error>
{
    let mut stream = orig_stream.clone();

    // Make sure the stream isn't exhausted
    stream.expect_current_exists("loop")?;

    // Must start with a while keyword
    stream.expect_and_consume(String::from("loop"))?;

    let statement = stream.accept_stream(parse_statement(&stream))?;

    
    Ok((stream, ParseTreeNode::Loop(vec![statement])))
}

/// Parse out a statement
fn parse_statement(orig_stream: &Stream)  -> Result<(Stream, ParseTreeNode), Error>
{
    let mut stream = orig_stream.clone();

    // Make sure the stream isn't exhausted
    stream.expect_current_exists("statement")?;

    // Simplest example of a statement is just a semicolon
    if stream.check_current(String::from(";"))
    {
        stream.consume();
        Ok((stream, ParseTreeNode::Statement(vec![])))
    }
    // The next simplest is the compound statement
    else if stream.check_current(String::from("{"))
    {
        stream.consume();
        
        let mut statements = vec![];

        while !stream.check_current(String::from("}"))
        {
            statements.push(stream.accept_stream(parse_statement(&stream))?);
        }

        stream.expect_and_consume(String::from("}"))?;

        Ok((stream, ParseTreeNode::Statements(statements)))
    }
    // Continue and break commands
    else if stream.check_current(String::from("continue")) || stream.check_current(String::from("break"))
    {
        let items = vec![stream.accept_stream(parse_token(&stream, "command"))?];

        stream.expect_and_consume(String::from(";"))?;

        Ok((stream, ParseTreeNode::Statement(items)))
    }
    // Initialization statement
    else if let Ok(val) = parse_type(&stream)
    {
        let datatype = stream.accept_stream(Ok(val))?;
        
        let assignments = stream.accept_stream(parse_assignments(&stream))?;

        stream.expect_and_consume(String::from(";"))?;

        Ok((stream, ParseTreeNode::AssignmentStatement(vec![datatype, assignments])))
    }
    // If Statement
    else if stream.check_current(String::from("if"))
    {
        parse_if_statement(&stream)
    }
    // While Loop
    else if stream.check_current(String::from("while"))
    {
        parse_while_loop(&stream)
    }
    // Do While Loop
    else if stream.check_current(String::from("do"))
    {
        parse_do_while_loop(&stream)
    }
    // Loop
    else if stream.check_current(String::from("loop"))
    {
        parse_loop(&stream)
    }
    // Return statement
    else if stream.check_current(String::from("return"))
    {
        stream.expect_and_consume(String::from("return"))?;

        let expr = stream.accept_stream(parse_expression(&stream))?;

        stream.expect_and_consume(String::from(";"))?;

        Ok((stream, ParseTreeNode::ReturnStatement(vec![expr])))
    }
    // Expression Statement
    else
    {
        let expr = stream.accept_stream(parse_expression(&stream))?;

        stream.expect_and_consume(String::from(";"))?;

        Ok((stream, ParseTreeNode::Statement(vec![expr])))
    }
}

/// Parse out an argument
/// for example u8** argv
/// in otherwords, a type and an identifier
fn parse_argument(orig_stream: &Stream)  -> Result<(Stream, ParseTreeNode), Error>
{
    let mut stream = orig_stream.clone();

    // Make sure the stream isn't exhausted
    stream.expect_current_exists("argument")?;

    // First is getting the return type
    let datatype = stream.accept_stream(parse_type(&stream))?;
    let mut items = vec![datatype];

    let name = stream.accept_stream(parse_identifier(&stream))?;
    items.push(name);

    Ok((stream, ParseTreeNode::Argument(items)))
}

/// Parse out an argument list
/// for example the argument list i32 argc, u8** argv
fn parse_arguments(orig_stream: &Stream)  -> Result<(Stream, ParseTreeNode), Error>
{
    let mut stream = orig_stream.clone();

    // Purposeful removal, it doesn't make sense to say we are looking for arguments here, the error
    // message could get somewhat confusing
    //stream.expect_current_exists("arguments")?;

    // First is getting the return type
    let arg = stream.accept_stream(parse_argument(&stream))?;
    let mut items = vec![arg];

    while stream.check_current(String::from(","))
    {
        stream.consume();
        items.push(stream.accept_stream(parse_argument(&stream))?);
    }

    Ok((stream, ParseTreeNode::Arguments(items)))
}

/// Parse out a function
/// Specifically, the return type, the function name, the arguments and a statement
fn parse_function(orig_stream: &Stream)  -> Result<(Stream, ParseTreeNode), Error>
{
    let mut stream = orig_stream.clone();

    // Make sure the stream isn't exhausted
    stream.expect_current_exists("function")?;

    // First is getting the return type
    let return_type = stream.accept_stream(parse_type(&stream))?;
    let mut items = vec![return_type];

    // Next the function name
    let func_name = stream.accept_stream(parse_identifier(&stream))?;
    items.push(func_name);

    // Next there must be a '('
    stream.expect(String::from("("))?;
    stream.consume();

    // If the next token is a ')', there is no argument list
    if stream.check_current(String::from(")"))
    {
        items.push(ParseTreeNode::Empty);
    }
    // Otherwise, get the arguments list
    else
    {
        let arg_list = stream.accept_stream(parse_arguments(&stream))?;
        items.push(arg_list);
    }

    // Consume the ')'
    stream.expect(String::from(")"))?;
    stream.consume();

    // Finally, there should be a statement here
    let statement = stream.accept_stream(parse_statement(&stream))?;
    items.push(statement);

    Ok((stream, ParseTreeNode::Function(items)))
}

/// Parse out a library
fn parse_library(orig_stream: &Stream)  -> Result<(Stream, ParseTreeNode), Error>
{
    let mut stream = orig_stream.clone();

    let mut items = vec![];
    
    while stream.peek().is_some()
    {
        let func = stream.accept_stream(parse_function(&stream))?;
        items.push(func);
    }

    Ok((stream, ParseTreeNode::Library(items)))
}