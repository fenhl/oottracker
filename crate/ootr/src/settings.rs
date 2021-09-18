use {
    std::{
        borrow::Cow,
        collections::{
            HashSet,
            HashMap,
        },
        fmt,
        iter,
        ops::{
            BitAnd,
            RangeInclusive,
        },
    },
    itertools::Itertools as _,
    crate::Rando,
};

#[derive(Debug)]
pub enum KnowledgeTypeError {
    BoolConflict,
    BoolType,
    BoolUnknown,
    IntConflict,
    IntType,
    IntUnknown,
    ListConflict,
    ListType,
    StrConflict,
    StrInvalid,
    StrType,
    StrUnknown,
    UnknownSetting,
}

impl fmt::Display for KnowledgeTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BoolConflict => write!(f, "conflicting Boolean setting knowledge"),
            Self::BoolType => write!(f, "expected Boolean setting knowledge"),
            Self::BoolUnknown => write!(f, "tried to get value of Boolean setting but it's not yet known"),
            Self::IntConflict => write!(f, "conflicting integer setting knowledge"),
            Self::IntType => write!(f, "expected integer setting knowledge"),
            Self::IntUnknown => write!(f, "tried to get value of integer setting but it's not yet known"),
            Self::ListConflict => write!(f, "conflicting list setting knowledge"),
            Self::ListType => write!(f, "expected list setting knowledge"),
            Self::StrConflict => write!(f, "conflicting string setting knowledge"),
            Self::StrInvalid => write!(f, "supplied invalid value for string setting knowledge"),
            Self::StrType => write!(f, "expected string setting knowledge"),
            Self::StrUnknown => write!(f, "tried to get single value of string setting but it's not yet known"),
            Self::UnknownSetting => write!(f, "no such setting"),
        }
    }
}

pub trait KnowledgeType: Sized { //TODO use from/into specific as required and implement from/into any based on them? Probably less boilerplate in the long term
    fn from_any(val: &KnowledgeValue) -> Result<Self, KnowledgeTypeError>;
    fn into_any(self) -> KnowledgeValue;
    fn from_bool(val: Option<bool>) -> Result<Self, KnowledgeTypeError> { Self::from_any(&KnowledgeValue::Bool(val)) }
    fn into_bool(self) -> Result<Option<bool>, KnowledgeTypeError> { if let KnowledgeValue::Bool(val) = self.into_any() { Ok(val) } else { Err(KnowledgeTypeError::BoolType) } }
    fn from_int(val: RangeInclusive<u8>) -> Result<Self, KnowledgeTypeError> { Self::from_any(&KnowledgeValue::Int(val)) }
    fn into_int(self) -> Result<RangeInclusive<u8>, KnowledgeTypeError> { if let KnowledgeValue::Int(val) = self.into_any() { Ok(val) } else { Err(KnowledgeTypeError::IntType) } }
    fn from_string(val: &HashSet<Cow<'static, str>>) -> Result<Self, KnowledgeTypeError> { Self::from_any(&KnowledgeValue::String(val.clone())) }
    fn into_string(self) -> Result<HashSet<Cow<'static, str>>, KnowledgeTypeError> { if let KnowledgeValue::String(val) = self.into_any() { Ok(val) } else { Err(KnowledgeTypeError::StrType) } }
    fn from_list(val: &HashMap<Cow<'static, str>, bool>) -> Result<Self, KnowledgeTypeError> { Self::from_any(&KnowledgeValue::List(val.clone())) }
    fn into_list(self) -> Result<HashMap<Cow<'static, str>, bool>, KnowledgeTypeError> { if let KnowledgeValue::List(val) = self.into_any() { Ok(val) } else { Err(KnowledgeTypeError::ListType) } }
}

#[derive(Clone)]
pub enum KnowledgeValue {
    Bool(Option<bool>),
    Int(RangeInclusive<u8>),
    String(HashSet<Cow<'static, str>>),
    List(HashMap<Cow<'static, str>, bool>),
}

impl KnowledgeType for KnowledgeValue {
    fn from_any(val: &KnowledgeValue) -> Result<Self, KnowledgeTypeError> { Ok(val.clone()) }
    fn into_any(self) -> KnowledgeValue { self }
    fn from_bool(val: Option<bool>) -> Result<Self, KnowledgeTypeError> { Ok(Self::Bool(val)) }
    fn into_bool(self) -> Result<Option<bool>, KnowledgeTypeError> { if let KnowledgeValue::Bool(val) = self { Ok(val) } else { Err(KnowledgeTypeError::BoolType) } }
    fn from_int(val: RangeInclusive<u8>) -> Result<Self, KnowledgeTypeError> { Ok(Self::Int(val)) }
    fn into_int(self) -> Result<RangeInclusive<u8>, KnowledgeTypeError> { if let Self::Int(val) = self { Ok(val) } else { Err(KnowledgeTypeError::IntType) } }
    fn from_string(val: &HashSet<Cow<'static, str>>) -> Result<Self, KnowledgeTypeError> { Ok(Self::String(val.clone())) }
    fn into_string(self) -> Result<HashSet<Cow<'static, str>>, KnowledgeTypeError> { if let Self::String(val) = self { Ok(val) } else { Err(KnowledgeTypeError::StrType) } }
    fn from_list(val: &HashMap<Cow<'static, str>, bool>) -> Result<Self, KnowledgeTypeError> { Ok(Self::List(val.clone())) }
    fn into_list(self) -> Result<HashMap<Cow<'static, str>, bool>, KnowledgeTypeError> { if let Self::List(val) = self { Ok(val) } else { Err(KnowledgeTypeError::ListType) } }
}

impl BitAnd for KnowledgeValue {
    type Output = Result<Self, KnowledgeTypeError>;

    fn bitand(mut self, rhs: Self) -> Result<Self, KnowledgeTypeError> {
        match &mut self {
            KnowledgeValue::Bool(b1) => if let KnowledgeValue::Bool(b2) = rhs {
                match (b1, b2) {
                    (Some(true), Some(false)) | (Some(false), Some(true)) => return Err(KnowledgeTypeError::BoolConflict),
                    (Some(_), None) => {}
                    (b1, _) => *b1 = b2,
                }
            } else {
                return Err(KnowledgeTypeError::BoolType)
            },
            KnowledgeValue::Int(i1) => if let KnowledgeValue::Int(i2) = rhs {
                *i1 = *i1.start().max(i2.start())..=*i1.end().min(i2.end());
                if i1.end() < i1.start() { return Err(KnowledgeTypeError::IntConflict) }
            } else {
                return Err(KnowledgeTypeError::IntType)
            },
            KnowledgeValue::String(s1) => if let KnowledgeValue::String(s2) = rhs {
                s1.retain(|val| s2.contains(val));
                if s1.is_empty() { return Err(KnowledgeTypeError::StrConflict) }
            } else {
                return Err(KnowledgeTypeError::StrType)
            },
            KnowledgeValue::List(l1) => if let KnowledgeValue::List(l2) = rhs {
                for (key, val2) in l2 {
                    if l1.get(&key).map_or(false, |&val1| val1 != val2) { return Err(KnowledgeTypeError::ListConflict) }
                    l1.insert(key, val2);
                }
            } else {
                return Err(KnowledgeTypeError::ListType)
            },
        }
        Ok(self)
    }
}

impl KnowledgeType for bool {
    fn from_any(val: &KnowledgeValue) -> Result<Self, KnowledgeTypeError> {
        if let &KnowledgeValue::Bool(opt_bool) = val {
            if let Some(b) = opt_bool {
                Ok(b)
            } else {
                Err(KnowledgeTypeError::BoolUnknown)
            }
        } else {
            Err(KnowledgeTypeError::BoolType)
        }
    }

    fn into_any(self) -> KnowledgeValue {
        KnowledgeValue::Bool(Some(self))
    }
}

impl KnowledgeType for u8 {
    fn from_any(val: &KnowledgeValue) -> Result<Self, KnowledgeTypeError> {
        if let KnowledgeValue::Int(range) = val {
            if range.start() == range.end() {
                Ok(*range.start())
            } else {
                Err(KnowledgeTypeError::IntUnknown)
            }
        } else {
            Err(KnowledgeTypeError::IntType)
        }
    }

    fn into_any(self) -> KnowledgeValue {
        KnowledgeValue::Int(self..=self)
    }
}

impl KnowledgeType for Cow<'static, str> {
    fn from_any(val: &KnowledgeValue) -> Result<Self, KnowledgeTypeError> {
        if let KnowledgeValue::String(strings) = val {
            if let Ok(s) = strings.iter().exactly_one() {
                Ok(s.clone())
            } else {
                Err(KnowledgeTypeError::StrUnknown)
            }
        } else {
            Err(KnowledgeTypeError::StrType)
        }
    }

    fn into_any(self) -> KnowledgeValue {
        KnowledgeValue::String(iter::once(self).collect())
    }
}

impl KnowledgeType for HashSet<Cow<'static, str>> {
    fn from_any(val: &KnowledgeValue) -> Result<Self, KnowledgeTypeError> {
        if let KnowledgeValue::String(strings) = val {
            Ok(strings.clone())
        } else {
            Err(KnowledgeTypeError::StrType)
        }
    }

    fn into_any(self) -> KnowledgeValue {
        KnowledgeValue::String(self)
    }
}

impl KnowledgeType for HashMap<Cow<'static, str>, bool> {
    fn from_any(val: &KnowledgeValue) -> Result<Self, KnowledgeTypeError> {
        if let KnowledgeValue::List(elts) = val {
            Ok(elts.clone())
        } else {
            Err(KnowledgeTypeError::ListType)
        }
    }

    fn into_any(self) -> KnowledgeValue {
        KnowledgeValue::List(self)
    }
}

pub trait Knowledge<R: Rando>: Sized {
    fn default(rando: &R) -> Result<Self, R::Err>;
    fn vanilla(rando: &R) -> Self;

    fn get<T: KnowledgeType>(&self, setting: &str) -> Result<Option<T>, KnowledgeTypeError>;

    /// Combines `self`'s knowledge for the given setting with the given value.
    fn update<T: KnowledgeType>(&mut self, setting: &str, value: T) -> Result<(), KnowledgeTypeError>;

    fn remove(&mut self, setting: &str);
}
