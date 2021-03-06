use crate::cli::Error;

use crate::irgen::{Function, DataType, NonPtrType, Symbol, Value, OpCode, get_value_type};

use super::{convert_to_llvm, bytes_size_of};

use std::collections::HashMap;

/// A value held in llvm
pub struct LLVMValue
{
    pub ptr: String,
    pub datatype: DataType
}

impl LLVMValue
{
    /// Generate a new LLVM Value object
    pub fn new(ptr: String, datatype: DataType) -> Self
    {
        Self
        {
            ptr,
            datatype
        }
    }

    /// Retrieve the datatype of the object
    pub fn get_datatype(&self) -> DataType
    {
        self.datatype
    }

    /// Retrieve the datatype of the pointer
    pub fn get_pointer_datatype(&self) -> DataType
    {
        let mut datatype = self.get_datatype().clone();
        datatype.num_ptr += 1;

        datatype
    }
}

/// A wrapper for giving a context to code generation for an LLVM function
pub struct FunctionGenerationContext
{
    func: Function,
    values: HashMap<String, LLVMValue>,
    next_temp: usize,
    result: String,
    current_arguments: String,
}

impl FunctionGenerationContext
{
    /// Generate a new function generation context object
    pub fn new(func: Function) -> Self
    {
        Self
        {
            func,
            values: HashMap::new(),
            next_temp: 0,
            result: String::new(),
            current_arguments: String::new()
        }
    }

    /// Insert a new command
    pub fn insert_command(&mut self, cmd: &str)
    {
        self.result += &format!("    {}\n", cmd);
    }

    /// Insert a label
    pub fn insert_label(&mut self, label: &str)
    {
        let l = self.render_value(&Value::Label(String::from(label)), false);
        self.insert_command(&format!("br {}", l));
        
        self.result += &format!("\n  {}:\n", label);
    }

    /// Get the next temporary variable
    pub fn get_next_temp(&mut self) -> String
    {
        self.next_temp += 1;
        format!("%V{}", self.next_temp - 1)
    }

    /// Create a new value
    pub fn create_new_value(&mut self, title: String, datatype: DataType)
    {
        // Create the raw object
        let ptr = self.get_next_temp();
        let new_value = LLVMValue::new(ptr, datatype);
        self.values.insert(title.clone(), new_value);

        let dt = self.values.get(&title).unwrap().get_datatype();
        let ptr = self.values.get(&title).unwrap().ptr.clone();

        // Add the command
        self.insert_command(
            &format!("{} = alloca {}, align {}", 
                            ptr, 
                            convert_to_llvm(&dt), 
                            bytes_size_of(&dt)));
    }

    /// Get the reference for a variable
    pub fn get_reference(&mut self, var: &Symbol, include_type: bool) -> String
    {
        // If the variable is not already stored, create it
        if !self.values.contains_key(&var.title)
        {
            self.create_new_value(var.title.clone(), var.datatype);
        }
        let ptr = self.values.get(&var.title).unwrap().ptr.clone();
        let pdt = self.values.get(&var.title).unwrap().get_pointer_datatype();

        if include_type
        {
            format!("{} {}",convert_to_llvm(&pdt), ptr)
        }
        else
        {
            ptr
        }
    }

    /// Get the value for a variable
    pub fn get_value(&mut self, var: &Symbol, include_type: bool) -> String
    {
        // If the variable is not already stored, create it
        if !self.values.contains_key(&var.title)
        {
            self.create_new_value(var.title.clone(), var.datatype);
        }
        
        let reg = self.get_next_temp();
        let dt = self.values.get(&var.title).unwrap().get_datatype();
        let pdt = self.values.get(&var.title).unwrap().get_pointer_datatype();
        let ptr = self.values.get(&var.title).unwrap().ptr.clone();

        self.insert_command(&format!("{} = load {}, {} {}, align {}", 
                                        reg, 
                                        convert_to_llvm(&dt),
                                        convert_to_llvm(&pdt),
                                        ptr,
                                        bytes_size_of(&var.datatype)));

        if include_type
        {
            if !(dt.raw_type == NonPtrType::Void && dt.num_ptr == 0)
            {
                format!("{} {}", convert_to_llvm(&dt), reg)
            }
            else
            {
                format!("{}", convert_to_llvm(&dt))
            }
        }
        else
        {
            reg
        }
    }

    /// Render a value for direct insertion into a command
    pub fn render_value(&mut self, val: &Value, include_type: bool) -> String
    {
        match val
        {
            Value::Label(label) =>
            {
                format!("label %{}", label)
            },
            Value::Literal(literal) =>
            {
                // If the type isn't void
                if !(literal.datatype.raw_type == NonPtrType::Void && literal.datatype.num_ptr == 0)
                {
                    if literal.datatype.num_ptr == 0 && !literal.datatype.is_ref
                    {
                        if include_type
                        {
                            format!("{} {}", convert_to_llvm(&literal.datatype), literal.value)
                        }
                        else
                        {
                            format!("{}", literal.value)
                        }
                    }
                    else
                    {
                        if include_type
                        {
                            format!("{0} inttoptr (i64 {1} to {0})", convert_to_llvm(&literal.datatype), literal.value)
                        }
                        else
                        {
                            format!("inttoptr (i64 {1} to {0})", convert_to_llvm(&literal.datatype), literal.value)
                        }
                    }
                }
                else
                {
                    if include_type
                    {
                        format!("{}", convert_to_llvm(&literal.datatype))
                    }
                    else
                    {
                        format!("")
                    }
                }
            },
            Value::Symbol(symbol) =>
            {
                self.get_value(symbol, include_type)
            }
        }
    }

    /// Render a pointer to a value for direct insertion into a command
    pub fn render_pointer(&mut self, val: &Value) -> String
    {
        match val
        {
            Value::Label(_) =>
            {
                panic!("The pointer of a label?!")
            },
            Value::Literal(_) =>
            {
                panic!("The pointer of a literal?!")
            },
            Value::Symbol(symbol) =>
            {
                self.get_reference(symbol, true)
            }
        }
    }

    /// Add a move via the syntax of the 'store' command
    pub fn add_move(&mut self, dest: &Value, src: String)
    {
        if let Some(datatype) = get_value_type(dest)
        {
            // If the data type is not a reference, just store the value into a pointer to the first
            if !datatype.is_ref
            {
                let val0 = self.render_pointer(dest);
                self.insert_command(
                            &format!("store {}, {}", 
                                        src,
                                        val0));
            }
            // Otherwise, the target *is* the pointer
            else
            {
                let val0 = self.render_value(dest, true);
                self.insert_command(
                            &format!("store {}, {}", 
                                        src,
                                        val0));
            }
        }
    }

    /// Add a compare command
    pub fn add_compare(&mut self, command: String, dest: String, src0: &Value, src1: &Value)
    {
        let val0 = self.render_value(src0, true);
        let val1 = self.render_value(src1, false);

        self.insert_command(&format!("{} = icmp {} {}, {}", dest,  command, val0, val1));
    }

    /// Render an IR function in LLVM IR
    pub fn render_function(&mut self) -> Result<String, Error>
    {
        // Clone the function to avoid borrow issues later
        let func = self.func.clone();

        self.result = String::new();

        // Function return type and name
        self.result += &format!("define {} @{}", convert_to_llvm(&func.return_type), func.name);

        // Arguments
        self.result += "(";

        let mut argument_names = vec![];

        for (i, (name, datatype)) in func.arguments.iter().enumerate()
        {
            let s = format!("{} %{}", convert_to_llvm(datatype), name);
            self.result += &s;

            if i < func.arguments.len() - 1
            {
                self.result += ", ";
            }

            argument_names.push((name.clone(), s));
        }

        self.result += ")\n";

        // Body

        self.result += "{\n";

        // Allocate all of the space required for the symbols
        for symbol in func.get_all_symbols()
        {
            if symbol.datatype.raw_type == NonPtrType::Void
            {
                continue;
            }

            self.create_new_value(symbol.title.clone(), symbol.datatype);

            // If the symbol is an argument, load the argument into the value
            for arg in &argument_names
            {
                if symbol.title.clone() == arg.0
                {
                    self.add_move(&Value::Symbol(symbol.clone()), arg.1.clone());
                }
            }
        }

        // Go over every instruction
        for i in 0..self.func.instructions.len()
        {
            let labels = if let Some(l) = func.labels.get(&i).clone() {l.clone()} else {vec![]};
            
            for label in labels
            {
                self.insert_label(label.as_str());
            }

            /* TODO:
                Ref*/

            if let Some(inst) = &func.instructions.get(&i)
            {
                self.result += &format!("\n; {}\n", inst);

                match &inst.opcode
                {
                    // Return Command
                    OpCode::Ret =>
                    {
                        let val = self.render_value(&inst.arguments[0], true).clone();
                        self.insert_command(&format!("ret {}", val));
                    },
                    // Move or Allocate
                    OpCode::Mov | OpCode::Alloc =>
                    {
                        let val = self.render_value(&inst.arguments[1], true);
                        self.add_move(&inst.arguments[0], val);
                    },
                    // Cast
                    OpCode::Cast =>
                    {
                        // Extract the types
                        let dest_type = get_value_type(&inst.arguments[0]).unwrap();
                        let src_type = get_value_type(&inst.arguments[1]).unwrap();

                        // Get the sizes of the types
                        let dest_size = bytes_size_of(&dest_type);
                        let src_size = bytes_size_of(&src_type);

                        let mut current = self.render_value(&inst.arguments[1], false);

                        let mut current_type = convert_to_llvm(&src_type);

                        if src_type.num_ptr > 0
                        {
                            let next = self.get_next_temp();
                            self.insert_command(&format!("{} = ptrtoint {} {} to i64", next, current_type, current));
                            current = next;
                            current_type = String::from("i64");
                        }

                        if convert_to_llvm(&dest_type) != convert_to_llvm(&src_type)
                        {
                            // If the destination is smaller, truncation is necessary
                            if dest_size < src_size && current_type != if dest_type.num_ptr == 0 {convert_to_llvm(&dest_type)} else {String::from("i64")}
                            {
                                let next = self.get_next_temp();
                                let next_type = if dest_type.num_ptr == 0 {convert_to_llvm(&dest_type)} else {String::from("i64")};
                                self.insert_command(&format!("{} = trunc {} {} to {}", next, current_type, current, next_type));
                                
                                current = next;
                                current_type = next_type;
                            }
                            // If the destination is larger, extension is necessary
                            else if dest_size > src_size && current_type != if dest_type.num_ptr == 0 {convert_to_llvm(&dest_type)} else {String::from("i64")}
                            {
                                let next = self.get_next_temp();
                                let next_type = if dest_type.num_ptr == 0 {convert_to_llvm(&dest_type)} else {String::from("i64")};
                                self.insert_command(&format!("{} = {} {} {} to {}", 
                                    next, if dest_type.is_signed() {"sext"} else {"zext"},
                                    current_type, current, next_type));

                                current = next;
                                current_type = next_type;
                            }

                            if dest_type.num_ptr > 0
                            {
                                // The source is not a pointer
                                if src_type.num_ptr == 0
                                {
                                    let next = self.get_next_temp();
                                    self.insert_command(&format!("{} = inttoptr {} {} to {}", next, current_type, current, convert_to_llvm(&dest_type)));
                                    current = next;
                                    current_type = convert_to_llvm(&dest_type);
                                }
                                // The source is a pointer
                                else
                                {
                                    let next = self.get_next_temp();
                                    self.insert_command(&format!("{} = bitcast {} {} to {}", next, current_type, current, convert_to_llvm(&dest_type)));
                                    current = next;
                                    current_type = convert_to_llvm(&dest_type);
                                }
                            }
                        }

                        self.add_move(&inst.arguments[0], format!("{} {}", current_type, current));
                    },
                    // Dereference Command
                    OpCode::Deref =>
                    {
                        if let Value::Symbol(var) = &inst.arguments[0]
                        {
                            let reg = self.get_next_temp();
                            let dt = self.values.get(&var.title).unwrap().get_datatype();

                            let val = self.render_value(&inst.arguments[1], true);

                            self.insert_command(&format!("{} = load {}, {}, align {}", 
                                            reg, 
                                            convert_to_llvm(&dt),
                                            val,
                                            bytes_size_of(&var.datatype)));

                            self.add_move(&inst.arguments[0], format!("{} {}", convert_to_llvm(&dt), reg));
                        };
                    },
                    // Dereference Command
                    OpCode::Ref =>
                    {
                        if let Value::Symbol(var0) = &inst.arguments[0]
                        {
                            if let Value::Symbol(var1) = &inst.arguments[1]
                            {
                                let ptr_dt = self.values.get(&var0.title).unwrap().get_pointer_datatype();
                                let ptr = self.values.get(&var1.title).unwrap().ptr.clone();

                                self.add_move(&inst.arguments[0], format!("{} {}", convert_to_llvm(&ptr_dt), ptr));
                            }
                        };
                    },
                    // Compare Commands
                    OpCode::Cne | OpCode::Ceq | OpCode::Cge | OpCode::Cgt | OpCode::Cle | OpCode::Clt =>
                    {
                        let temp = self.get_next_temp();
                        let temp2 = self.get_next_temp();

                        let is_signed = get_value_type(&inst.arguments[1]).unwrap().is_signed();

                        let command = String::from(
                            match &inst.opcode
                            {
                                OpCode::Cne => "ne",
                                OpCode::Ceq => "eq",
                                OpCode::Cge => if is_signed {"sge"} else {"uge"},
                                OpCode::Cle => if is_signed {"sle"} else {"ule"},
                                OpCode::Cgt => if is_signed {"sgt"} else {"ugt"},
                                OpCode::Clt => if is_signed {"slt"} else {"ult"}
                                _ => panic!()
                            }
                        );

                        self.add_compare(command, temp.clone(), &inst.arguments[1], &inst.arguments[2]);
                        self.insert_command(&format!("{} = zext i1 {} to {}", &temp2, &temp, convert_to_llvm(&get_value_type(&inst.arguments[0]).unwrap())));
                        self.add_move(&inst.arguments[0], format!("{} {}", convert_to_llvm(&get_value_type(&inst.arguments[0]).unwrap()), temp2));
                    },
                    // Branch Commands
                    OpCode::Bne | OpCode::Beq | OpCode::Bge | OpCode::Bgt | OpCode::Ble | OpCode::Blt =>
                    {
                        let temp = self.get_next_temp();

                        let label_true = self.render_value(&inst.arguments[2], true);
                        let label_false = self.render_value(&inst.arguments[3], true);

                        let is_signed = get_value_type(&inst.arguments[0]).unwrap().is_signed();

                        let command = String::from(
                            match &inst.opcode
                            {
                                OpCode::Bne => "ne",
                                OpCode::Beq => "eq",
                                OpCode::Bge => if is_signed {"sge"} else {"uge"},
                                OpCode::Ble => if is_signed {"sle"} else {"ule"},
                                OpCode::Bgt => if is_signed {"sgt"} else {"ugt"},
                                OpCode::Blt => if is_signed {"slt"} else {"ult"}
                                _ => panic!()
                            }
                        );

                        self.add_compare(command, temp.clone(), &inst.arguments[0], &inst.arguments[1]);
                        self.insert_command(&format!("br i1 {}, {}, {}", &temp, label_true, label_false));
                    },
                    // Add Command
                    OpCode::Add =>
                    {
                        let temp = self.get_next_temp();

                        let val0 = self.render_value(&inst.arguments[1], true);
                        let val1 =  self.render_value(&inst.arguments[2], false);

                        self.insert_command(&format!("{} = add {}, {}", temp, val0, val1));
                        self.add_move(&inst.arguments[0], format!("{} {}", convert_to_llvm(&get_value_type(&inst.arguments[0]).unwrap()), temp));
                    },
                    // Sub Command
                    OpCode::Sub =>
                    {
                        let temp = self.get_next_temp();

                        let val0 = self.render_value(&inst.arguments[1], true);
                        let val1 =  self.render_value(&inst.arguments[2], false);

                        self.insert_command(&format!("{} = sub {}, {}", temp, val0, val1));
                        self.add_move(&inst.arguments[0], format!("{} {}", convert_to_llvm(&get_value_type(&inst.arguments[0]).unwrap()), temp));
                    },
                    // Mul Command
                    OpCode::Mul =>
                    {
                        let temp = self.get_next_temp();

                        let val0 = self.render_value(&inst.arguments[1], true);
                        let val1 =  self.render_value(&inst.arguments[2], false);

                        self.insert_command(&format!("{} = mul {}, {}", temp, val0, val1));
                        self.add_move(&inst.arguments[0], format!("{} {}", convert_to_llvm(&get_value_type(&inst.arguments[0]).unwrap()), temp));
                    },
                    // Div Command
                    OpCode::Div =>
                    {
                        let temp = self.get_next_temp();

                        let val0 = self.render_value(&inst.arguments[1], true);
                        let val1 =  self.render_value(&inst.arguments[2], false);

                        self.insert_command(&format!("{} = div {}, {}", temp, val0, val1));
                        self.add_move(&inst.arguments[0], format!("{} {}", convert_to_llvm(&get_value_type(&inst.arguments[0]).unwrap()), temp));
                    },
                    // And Command
                    OpCode::And =>
                    {
                        let temp = self.get_next_temp();

                        let val0 = self.render_value(&inst.arguments[1], true);
                        let val1 =  self.render_value(&inst.arguments[2], false);

                        self.insert_command(&format!("{} = and {}, {}", temp, val0, val1));
                        self.add_move(&inst.arguments[0], format!("{} {}", convert_to_llvm(&get_value_type(&inst.arguments[0]).unwrap()), temp));
                    },
                    // Or Command
                    OpCode::Or =>
                    {
                        let temp = self.get_next_temp();

                        let val0 = self.render_value(&inst.arguments[1], true);
                        let val1 =  self.render_value(&inst.arguments[2], false);

                        self.insert_command(&format!("{} = or {}, {}", temp, val0, val1));
                        self.add_move(&inst.arguments[0], format!("{} {}", convert_to_llvm(&get_value_type(&inst.arguments[0]).unwrap()), temp));
                    },
                    // Xor Command
                    OpCode::Xor =>
                    {
                        let temp = self.get_next_temp();

                        let val0 = self.render_value(&inst.arguments[1], true);
                        let val1 =  self.render_value(&inst.arguments[2], false);

                        self.insert_command(&format!("{} = xor {}, {}", temp, val0, val1));
                        self.add_move(&inst.arguments[0], format!("{} {}", convert_to_llvm(&get_value_type(&inst.arguments[0]).unwrap()), temp));
                    },
                    // Mod Command
                    OpCode::Mod =>
                    {
                        let temp = self.get_next_temp();

                        let val0 = self.render_value(&inst.arguments[1], true);
                        let val1 =  self.render_value(&inst.arguments[2], false);

                        self.insert_command(&format!("{} = {} {}, {}", temp, if get_value_type(&inst.arguments[1]).unwrap().is_signed() {"srem"} else {"urem"}, val0, val1));
                        self.add_move(&inst.arguments[0], format!("{} {}", convert_to_llvm(&get_value_type(&inst.arguments[0]).unwrap()), temp));
                    },
                    // Shl Command
                    OpCode::Shl =>
                    {
                        let temp = self.get_next_temp();

                        let val0 = self.render_value(&inst.arguments[1], true);
                        let val1 =  self.render_value(&inst.arguments[2], false);

                        self.insert_command(&format!("{} = shl {}, {}", temp, val0, val1));
                        self.add_move(&inst.arguments[0], format!("{} {}", convert_to_llvm(&get_value_type(&inst.arguments[0]).unwrap()), temp));
                    },
                    // Shr Command
                    OpCode::Shr =>
                    {
                        let temp = self.get_next_temp();

                        let val0 = self.render_value(&inst.arguments[1], true);
                        let val1 =  self.render_value(&inst.arguments[2], false);

                        self.insert_command(&format!("{} = {} {}, {}", temp, if get_value_type(&inst.arguments[1]).unwrap().is_signed() {"lshr"} else {"ashr"}, val0, val1));
                        self.add_move(&inst.arguments[0], format!("{} {}", convert_to_llvm(&get_value_type(&inst.arguments[0]).unwrap()), temp));
                    },
                    // Array Command
                    OpCode::Array =>
                    {
                        let temp = self.get_next_temp();
                        let temp2 = self.get_next_temp();

                        let val0 = self.render_value(&inst.arguments[1], true);
                        let val1 =  self.render_value(&inst.arguments[2], true);

                        let val_type = convert_to_llvm(&get_value_type(&inst.arguments[0]).unwrap());
                        let ptr_type = convert_to_llvm(&get_value_type(&inst.arguments[1]).unwrap());

                        self.insert_command(&format!("{} = getelementptr {}, {}, {}", temp, val_type, val0, val1));

                        self.insert_command(&format!("{} = load {}, {} {}, align {}", temp2, val_type, ptr_type, temp,
                                            bytes_size_of(&get_value_type(&inst.arguments[0]).unwrap())));

                        self.add_move(&inst.arguments[0], format!("{} {}", convert_to_llvm(&get_value_type(&inst.arguments[0]).unwrap()), temp2));
                    },
                    // Push Command
                    OpCode::Push =>
                    {
                        let arg = self.render_value(&inst.arguments[0], true);
                        self.current_arguments += &format!("{}, ", arg);
                    },
                    // Call Command
                    OpCode::Call =>
                    {
                        let temp = self.get_next_temp();
                        if let Value::Label(func_label) = &inst.arguments[1]
                        {
                            if self.current_arguments.len() > 0
                            {
                                self.current_arguments.pop();
                                self.current_arguments.pop();
                            }

                            self.insert_command(&format!("{} = call {} @{}({})",
                                                    temp, 
                                                    convert_to_llvm(&get_value_type(&inst.arguments[0]).unwrap()),
                                                    func_label,
                                                    self.current_arguments));

                            self.current_arguments = String::new();

                            self.add_move(&inst.arguments[0], format!("{} {}", convert_to_llvm(&get_value_type(&inst.arguments[0]).unwrap()), temp));
                        }
                    },
                    // Unconditional Jump
                    OpCode::Jmp =>
                    {
                        let label = self.render_value(&inst.arguments[0], true);
                        self.insert_command(&format!("br {}", label));
                    },
                    // This should never happen, but if it does, ignore it
                    OpCode::Nop => {}
                }
            }
        }

        self.result += "}\n";

        Ok(self.result.clone())
    }
}