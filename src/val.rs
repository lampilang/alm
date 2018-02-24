use std::sync::Arc;

pub trait MultiTyped {
    fn type_code(&self) -> Type;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    Nil = 0,
    Byte = 1,
    IWord = 2,
    UWord = 3,
    I64 = 4,
    U64 = 5,
    F64 = 6,
    Array = 7,
    Tuple = 8,
    Function = 9,
    Extension = 10,
}

#[derive(Clone, Debug)]
pub enum Value {
    Nil(),
    Byte(u8),
    IWord(isize),
    UWord(usize),
    I64(i64),
    U64(u64),
    F64(f64),
    Array(Array),
    Tuple(Tuple),
    Function(Function),
    Extension(*mut ()),
}

impl MultiTyped for Value {

    fn type_code(&self) -> Type {
        match *self {
            Value::Nil() => Type::Nil,
            Value::Byte(_) => Type::Byte,
            Value::IWord(_) => Type::IWord,
            Value::UWord(_) => Type::UWord,
            Value::I64(_) => Type::I64,
            Value::U64(_) => Type::U64,
            Value::F64(_) => Type::F64,
            Value::Array(_) => Type::Array,
            Value::Tuple(_) => Type::Tuple,
            Value::Function(_) => Type::Function,
            Value::Extension(_) => Type::Extension,
        }
    }

}

#[derive(Clone, Debug)]
pub enum ArrayType {
    Nil(usize),
    Byte(Arc<Vec<u8>>),
    IWord(Arc<Vec<isize>>),
    UWord(Arc<Vec<usize>>),
    I64(Arc<Vec<i64>>),
    U64(Arc<Vec<u64>>),
    F64(Arc<Vec<f64>>),
    Array(Arc<Vec<Array>>),
    Tuple(Arc<Vec<Tuple>>),
    Function(Arc<Vec<Function>>),
    Extension(Arc<Vec<*mut ()>>),
}

impl MultiTyped for ArrayType {

    fn type_code(&self) -> Type {
        match *self {
            ArrayType::Nil(_) => Type::Nil,
            ArrayType::Byte(_) => Type::Byte,
            ArrayType::IWord(_) => Type::IWord,
            ArrayType::UWord(_) => Type::UWord,
            ArrayType::I64(_) => Type::I64,
            ArrayType::U64(_) => Type::U64,
            ArrayType::F64(_) => Type::F64,
            ArrayType::Array(_) => Type::Array,
            ArrayType::Tuple(_) => Type::Tuple,
            ArrayType::Function(_) => Type::Function,
            ArrayType::Extension(_) => Type::Extension,
        }
    }

}

#[derive(Clone, Debug)]
pub struct Array {
    memory: Arc<ArrayInner>
}

#[derive(Clone, Debug)]
pub struct Tuple {
    memory: Arc<TupleInner>
}

#[derive(Clone, Debug)]
pub struct Function {
    memory: Arc<FunctionInner>
}

#[derive(Clone, Debug)]
struct FunctionInner {
    env: Value,
    bc: Arc<Vec<u8>>,
    name: Arc<Vec<u8>>,
}

#[derive(Clone, Debug)]
struct ArrayInner {
    typ: ArrayType,
}

#[derive(Clone, Debug)]
struct TupleInner {
    elems: Vec<Value>,
}
