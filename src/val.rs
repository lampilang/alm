use edi::Edi;
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
    Edi = 10,
    Extension = 11,
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
    Edi(Edi),
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
            Value::Edi(_) => Type::Edi,
            Value::Extension(_) => Type::Extension,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ArrayType {
    Nil(usize),
    Byte(Arc<[u8]>),
    IWord(Arc<[isize]>),
    UWord(Arc<[usize]>),
    I64(Arc<[i64]>),
    U64(Arc<[u64]>),
    F64(Arc<[f64]>),
    Array(Arc<[Array]>),
    Tuple(Arc<[Tuple]>),
    Function(Arc<[Function]>),
    Edi(Arc<[Edi]>),
    Extension(Arc<[*mut ()]>),
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
            ArrayType::Edi(_) => Type::Edi,
            ArrayType::Extension(_) => Type::Extension,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Array {
    inner: Arc<ArrayType>,
}

impl Array {
    pub fn type_enum(&self) -> &Arc<ArrayType> { &self.inner }

    pub fn new(typ: Arc<ArrayType>) -> Self { Self { inner: typ } }
}

#[derive(Clone, Debug)]
pub struct Tuple {
    inner: Arc<[Value]>,
}

impl Tuple {
    pub fn elements(&self) -> &Arc<[Value]> { &self.inner }

    pub fn new(elems: Arc<[Value]>) -> Self { Self { inner: elems } }
}

#[derive(Clone, Debug)]
pub struct Function {
    inner: Arc<FunctionInner>,
}

#[derive(Clone, Debug)]
struct FunctionInner {
    env: Value,
    bc: Arc<[u8]>,
    name: Arc<[u8]>,
}

impl Function {
    pub fn name(&self) -> &Arc<[u8]> { &self.inner.name }

    pub fn bc(&self) -> &Arc<[u8]> { &self.inner.bc }

    pub fn env(&self) -> &Value { &self.inner.env }

    pub fn new(env: Value, bc: Arc<[u8]>, name: Arc<[u8]>) -> Self {
        Self {
            inner: Arc::new(FunctionInner { env, bc, name }),
        }
    }
}
