// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::location::*;
use crate::shared::{Address, Identifier, Name, TName};
use crate::sp;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt;
use core::hash::Hash;

macro_rules! new_name {
    ($n:ident) => {
        #[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Clone)]
        pub struct $n(pub Name);

        impl TName for $n {
            type Key = String;
            type Loc = Loc;

            fn drop_loc(self) -> (Loc, String) {
                (self.0.loc, self.0.value)
            }

            fn add_loc(loc: Loc, key: String) -> Self {
                $n(sp(loc, key))
            }

            fn borrow(&self) -> (&Loc, &String) {
                (&self.0.loc, &self.0.value)
            }
        }

        impl Identifier for $n {
            fn value(&self) -> &str {
                &self.0.value
            }
            fn loc(&self) -> Loc {
                self.0.loc
            }
        }

        impl fmt::Display for $n {
            fn fmt(&self, f: &mut fmt::Formatter) -> core::fmt::Result {
                write!(f, "{}", &self.0)
            }
        }
    };
}

//**************************************************************************************************
// Program
//**************************************************************************************************

#[derive(Debug, Clone)]
pub struct Program {
    pub source_definitions: Vec<Definition>,
    pub lib_definitions: Vec<Definition>,
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Definition {
    Module(ModuleDefinition),
    Address(Loc, Address, Vec<ModuleDefinition>),
    Script(Script),
}

#[derive(Debug, Clone)]
pub struct Script {
    pub loc: Loc,
    pub uses: Vec<Use>,
    pub constants: Vec<Constant>,
    pub function: Function,
    pub specs: Vec<SpecBlock>,
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum Use {
    Module(ModuleIdent, Option<ModuleName>),
    Members(ModuleIdent, Vec<(Name, Option<Name>)>),
}

//**************************************************************************************************
// Modules
//**************************************************************************************************

new_name!(ModuleName);

#[derive(Debug, Clone)]
pub struct ModuleIdent {
    pub locs: (
        /* whole entity loc */ Loc,
        /* module name loc */ Loc,
    ),
    pub value: (Address, String),
}

#[derive(Debug, Clone)]
pub struct ModuleDefinition {
    pub loc: Loc,
    pub name: ModuleName,
    pub members: Vec<ModuleMember>,
}

#[derive(Debug, Clone)]
pub enum ModuleMember {
    Function(Function),
    Struct(StructDefinition),
    Spec(SpecBlock),
    Use(Use),
    Friend(Friend),
    Constant(Constant),
}

//**************************************************************************************************
// Friends
//**************************************************************************************************

#[derive(Debug, PartialEq, Clone)]
pub enum Friend_ {
    Module(ModuleName),
    QualifiedModule(ModuleIdent),
}

pub type Friend = Spanned<Friend_>;

//**************************************************************************************************
// Structs
//**************************************************************************************************

new_name!(Field);
new_name!(StructName);

pub type ResourceLoc = Option<Loc>;

#[derive(Debug, PartialEq, Clone)]
pub struct StructDefinition {
    pub loc: Loc,
    pub abilities: Vec<Ability>,
    pub name: StructName,
    pub type_parameters: Vec<(Name, Vec<Ability>)>,
    pub fields: StructFields,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StructFields {
    Defined(Vec<(Field, Type)>),
    Native(Loc),
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

new_name!(FunctionName);

#[derive(PartialEq, Clone, Debug)]
pub struct FunctionSignature {
    pub type_parameters: Vec<(Name, Vec<Ability>)>,
    pub parameters: Vec<(Var, Type)>,
    pub return_type: Type,
}

#[derive(PartialEq, Debug, Clone)]
pub enum FunctionVisibility {
    Public(Loc),
    Script(Loc),
    Friend(Loc),
    Internal,
}

#[derive(PartialEq, Clone, Debug)]
pub enum FunctionBody_ {
    Defined(Sequence),
    Native,
}
pub type FunctionBody = Spanned<FunctionBody_>;

#[derive(PartialEq, Debug, Clone)]
// (public?) foo<T1(: copyable?), ..., TN(: copyable?)>(x1: t1, ..., xn: tn): t1 * ... * tn {
//    body
//  }
// (public?) native foo<T1(: copyable?), ..., TN(: copyable?)>(x1: t1, ..., xn: tn): t1 * ... * tn;
pub struct Function {
    pub loc: Loc,
    pub visibility: FunctionVisibility,
    pub signature: FunctionSignature,
    pub acquires: Vec<ModuleAccess>,
    pub name: FunctionName,
    pub body: FunctionBody,
}

//**************************************************************************************************
// Constants
//**************************************************************************************************

new_name!(ConstantName);

#[derive(PartialEq, Debug, Clone)]
pub struct Constant {
    pub loc: Loc,
    pub signature: Type,
    pub name: ConstantName,
    pub value: Exp,
}

//**************************************************************************************************
// Specification Blocks
//**************************************************************************************************

// Specification block:
//    SpecBlock = "spec" <SpecBlockTarget> "{" SpecBlockMember* "}"
#[derive(Debug, Clone, PartialEq)]
pub struct SpecBlock_ {
    pub target: SpecBlockTarget,
    pub uses: Vec<Use>,
    pub members: Vec<SpecBlockMember>,
}

pub type SpecBlock = Spanned<SpecBlock_>;

#[derive(Debug, Clone, PartialEq)]
pub enum SpecBlockTarget_ {
    Code,
    Module,
    Function(FunctionName),
    Structure(StructName),
    Schema(Name, Vec<(Name, Vec<Ability>)>),
}

pub type SpecBlockTarget = Spanned<SpecBlockTarget_>;

#[derive(Debug, Clone, PartialEq)]
pub struct PragmaProperty_ {
    pub name: Name,
    pub value: Option<PragmaValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PragmaValue {
    Literal(Value),
    Ident(ModuleAccess),
}

pub type PragmaProperty = Spanned<PragmaProperty_>;

#[derive(Debug, Clone, PartialEq)]
pub struct SpecApplyPattern_ {
    pub visibility: Option<FunctionVisibility>,
    pub name_pattern: Vec<SpecApplyFragment>,
    pub type_parameters: Vec<(Name, Vec<Ability>)>,
}

pub type SpecApplyPattern = Spanned<SpecApplyPattern_>;

#[derive(Debug, Clone, PartialEq)]
pub enum SpecApplyFragment_ {
    Wildcard,
    NamePart(Name),
}

pub type SpecApplyFragment = Spanned<SpecApplyFragment_>;

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum SpecBlockMember_ {
    Condition {
        kind: SpecConditionKind,
        properties: Vec<PragmaProperty>,
        exp: Exp,
        additional_exps: Vec<Exp>,
    },
    Function {
        uninterpreted: bool,
        name: FunctionName,
        signature: FunctionSignature,
        body: FunctionBody,
    },
    Variable {
        is_global: bool,
        name: Name,
        type_parameters: Vec<(Name, Vec<Ability>)>,
        type_: Type,
    },
    Let {
        name: Name,
        def: Exp,
    },
    Include {
        properties: Vec<PragmaProperty>,
        exp: Exp,
    },
    Apply {
        exp: Exp,
        patterns: Vec<SpecApplyPattern>,
        exclusion_patterns: Vec<SpecApplyPattern>,
    },
    Pragma {
        properties: Vec<PragmaProperty>,
    },
}

pub type SpecBlockMember = Spanned<SpecBlockMember_>;

// Specification condition kind.
#[derive(PartialEq, Clone, Debug)]
pub enum SpecConditionKind {
    Assert,
    Assume,
    Decreases,
    AbortsIf,
    AbortsWith,
    SucceedsIf,
    Modifies,
    Emits,
    Ensures,
    Requires,
    RequiresModule,
    Invariant,
    InvariantUpdate,
    InvariantPack,
    InvariantUnpack,
    InvariantModule,
}

// Specification invariant kind.
#[derive(Debug, PartialEq)]
pub enum InvariantKind {
    Data,
    Update,
    Pack,
    Unpack,
    Module,
}

//**************************************************************************************************
// Types
//**************************************************************************************************

// A ModuleAccess references a local or global name or something from a module,
// either a struct type or a function.
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleAccess_ {
    // N
    Name(Name),
    // M::S
    ModuleAccess(ModuleName, Name),
    // OxADDR::M::S
    QualifiedModuleAccess(ModuleIdent, Name),
}
pub type ModuleAccess = Spanned<ModuleAccess_>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Ability_ {
    Copy,
    Drop,
    Store,
    Key,
}
pub type Ability = Spanned<Ability_>;

#[derive(Debug, Clone, PartialEq)]
pub enum Type_ {
    // N
    // N<t1, ... , tn>
    Apply(Box<ModuleAccess>, Vec<Type>),
    // &t
    // &mut t
    Ref(bool, Box<Type>),
    // (t1,...,tn):t
    Fun(Vec<Type>, Box<Type>),
    // ()
    Unit,
    // (t1, t2, ... , tn)
    // Used for return values and expression blocks
    Multiple(Vec<Type>),
}
pub type Type = Spanned<Type_>;

//**************************************************************************************************
// Expressions
//**************************************************************************************************

new_name!(Var);

#[derive(Debug, Clone, PartialEq)]
pub enum Bind_ {
    // x
    Var(Var),
    // T { f1: b1, ... fn: bn }
    // T<t1, ... , tn> { f1: b1, ... fn: bn }
    Unpack(ModuleAccess, Option<Vec<Type>>, Vec<(Field, Bind)>),
}
pub type Bind = Spanned<Bind_>;
// b1, ..., bn
pub type BindList = Spanned<Vec<Bind>>;

pub type BindWithRange = Spanned<(Bind, Exp)>;
pub type BindWithRangeList = Spanned<Vec<BindWithRange>>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value_ {
    // 0x<hex representation up to 64 digits with padding 0s>
    Address(Address),
    // <num>u8
    U8(u8),
    // <num>u64
    U64(u64),
    // <num>u128
    U128(u128),
    // true
    // false
    Bool(bool),
    // x"[0..9A..F]+"
    HexString(String),
    // b"(<ascii> | \n | \r | \t | \\ | \0 | \" | \x[0..9A..F][0..9A..F])+"
    ByteString(String),
}
pub type Value = Spanned<Value_>;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum UnaryOp_ {
    // !
    Not,
}
pub type UnaryOp = Spanned<UnaryOp_>;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BinOp_ {
    // Int ops
    // +
    Add,
    // -
    Sub,
    // *
    Mul,
    // %
    Mod,
    // /
    Div,
    // |
    BitOr,
    // &
    BitAnd,
    // ^
    Xor,
    // <<
    Shl,
    // >>
    Shr,
    // ..
    Range, // spec only

    // Bool ops
    // ==>
    Implies, // spec only
    // &&
    And,
    // ||
    Or,

    // Compare Ops
    // ==
    Eq,
    // !=
    Neq,
    // <
    Lt,
    // >
    Gt,
    // <=
    Le,
    // >=
    Ge,
}
pub type BinOp = Spanned<BinOp_>;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum QuantKind_ {
    Forall,
    Exists,
}
pub type QuantKind = Spanned<QuantKind_>;

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Exp_ {
    Value(Value),
    // <num>
    InferredNum(u128),
    // move(x)
    Move(Var),
    // copy(x)
    Copy(Var),
    // [m::]n[<t1, .., tn>]
    Name(ModuleAccess, Option<Vec<Type>>),

    // f(earg,*)
    Call(ModuleAccess, Option<Vec<Type>>, Spanned<Vec<Exp>>),

    // tn {f1: e1, ... , f_n: e_n }
    Pack(ModuleAccess, Option<Vec<Type>>, Vec<(Field, Exp)>),

    // if (eb) et else ef
    IfElse(Box<Exp>, Box<Exp>, Option<Box<Exp>>),
    // while (eb) eloop
    While(Box<Exp>, Box<Exp>),
    // loop eloop
    Loop(Box<Exp>),

    // { seq }
    Block(Sequence),
    // fun (x1, ..., xn) e
    Lambda(BindList, Box<Exp>), // spec only
    // forall/exists x1 : e1, ..., xn [{ t1, .., tk } *] [where cond]: en.
    Quant(
        QuantKind,
        BindWithRangeList,
        Vec<Vec<Exp>>,
        Option<Box<Exp>>,
        Box<Exp>,
    ), // spec only
    // (e1, ..., en)
    ExpList(Vec<Exp>),
    // ()
    Unit,

    // a = e
    Assign(Box<Exp>, Box<Exp>),

    // return e
    Return(Option<Box<Exp>>),
    // abort e
    Abort(Box<Exp>),
    // break
    Break,
    // continue
    Continue,

    // *e
    Dereference(Box<Exp>),
    // op e
    UnaryExp(UnaryOp, Box<Exp>),
    // e1 op e2
    BinopExp(Box<Exp>, BinOp, Box<Exp>),

    // &e
    // &mut e
    Borrow(bool, Box<Exp>),

    // e.f
    Dot(Box<Exp>, Name),
    // e[e']
    Index(Box<Exp>, Box<Exp>), // spec only

    // (e as t)
    Cast(Box<Exp>, Type),
    // (e: t)
    Annotate(Box<Exp>, Type),

    // spec { ... }
    Spec(SpecBlock),

    // Internal node marking an error was added to the error list
    // This is here so the pass can continue even when an error is hit
    UnresolvedError,
}
pub type Exp = Spanned<Exp_>;

// { e1; ... ; en }
// { e1; ... ; en; }
// The Loc field holds the source location of the final semicolon, if there is one.
pub type Sequence = (Vec<Use>, Vec<SequenceItem>, Option<Loc>, Box<Option<Exp>>);
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum SequenceItem_ {
    // e;
    Seq(Box<Exp>),
    // let b : t = e;
    // let b = e;
    Declare(BindList, Option<Type>),
    // let b : t = e;
    // let b = e;
    Bind(BindList, Option<Type>, Box<Exp>),
}
pub type SequenceItem = Spanned<SequenceItem_>;

//**************************************************************************************************
// Traits
//**************************************************************************************************

impl TName for ModuleIdent {
    type Key = (Address, String);
    type Loc = (Loc, Loc);

    fn drop_loc(self) -> ((Loc, Loc), (Address, String)) {
        (self.locs, self.value)
    }

    fn add_loc(locs: (Loc, Loc), value: (Address, String)) -> ModuleIdent {
        ModuleIdent { locs, value }
    }

    fn borrow(&self) -> (&(Loc, Loc), &(Address, String)) {
        (&self.locs, &self.value)
    }
}

// Hash, Eq, PartialEq, Ord, PartialOrd,
impl PartialEq for ModuleIdent {
    fn eq(&self, other: &ModuleIdent) -> bool {
        self.value == other.value
    }
}

impl Eq for ModuleIdent {}

impl PartialOrd for ModuleIdent {
    fn partial_cmp(&self, other: &ModuleIdent) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for ModuleIdent {
    fn cmp(&self, other: &ModuleIdent) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl Hash for ModuleIdent {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state)
    }
}

impl TName for Ability {
    type Key = Ability_;
    type Loc = Loc;

    fn drop_loc(self) -> (Self::Loc, Self::Key) {
        let sp!(loc, ab_) = self;
        (loc, ab_)
    }

    fn add_loc(loc: Self::Loc, key: Self::Key) -> Self {
        sp(loc, key)
    }

    fn borrow(&self) -> (&Self::Loc, &Self::Key) {
        (&self.loc, &self.value)
    }
}

//**************************************************************************************************
// Impl
//**************************************************************************************************

impl Definition {
    pub fn file(&self) -> &'static str {
        match self {
            Definition::Module(m) => m.loc.file(),
            Definition::Address(loc, _, _) => loc.file(),
            Definition::Script(s) => s.loc.file(),
        }
    }
}

impl ModuleIdent {
    pub fn loc(&self) -> Loc {
        self.locs.0
    }
}

impl ModuleName {
    pub const SELF_NAME: &'static str = "Self";
}

impl Var {
    pub fn is_underscore(&self) -> bool {
        self.0.value == "_"
    }

    pub fn starts_with_underscore(&self) -> bool {
        self.0.value.starts_with('_')
    }
}

impl Ability_ {
    pub const COPY: &'static str = "copy";
    pub const DROP: &'static str = "drop";
    pub const STORE: &'static str = "store";
    pub const KEY: &'static str = "key";

    /// For a struct with ability `a`, each field needs to have the ability `a.requires()`.
    /// Consider a generic type Foo<t1, ..., tn>, for Foo<t1, ..., tn> to have ability `a`, Foo must
    /// have been declared with `a` and each type argument ti must have the ability `a.requires()`
    pub fn requires(self) -> Ability_ {
        match self {
            Ability_::Copy => Ability_::Copy,
            Ability_::Drop => Ability_::Drop,
            Ability_::Store => Ability_::Store,
            Ability_::Key => Ability_::Store,
        }
    }

    /// An inverse of `requires`, where x is in a.required_by() iff x.requires() == a
    pub fn required_by(self) -> Vec<Ability_> {
        match self {
            Self::Copy => vec![Ability_::Copy],
            Self::Drop => vec![Ability_::Drop],
            Self::Store => vec![Ability_::Store, Ability_::Key],
            Self::Key => vec![],
        }
    }
}

impl Type_ {
    pub fn unit(loc: Loc) -> Type {
        sp(loc, Type_::Unit)
    }
}

impl UnaryOp_ {
    pub const NOT: &'static str = "!";

    pub fn symbol(&self) -> &'static str {
        use UnaryOp_ as U;
        match self {
            U::Not => U::NOT,
        }
    }

    pub fn is_pure(&self) -> bool {
        use UnaryOp_ as U;
        match self {
            U::Not => true,
        }
    }
}

impl BinOp_ {
    pub const ADD: &'static str = "+";
    pub const SUB: &'static str = "-";
    pub const MUL: &'static str = "*";
    pub const MOD: &'static str = "%";
    pub const DIV: &'static str = "/";
    pub const BIT_OR: &'static str = "|";
    pub const BIT_AND: &'static str = "&";
    pub const XOR: &'static str = "^";
    pub const SHL: &'static str = "<<";
    pub const SHR: &'static str = ">>";
    pub const AND: &'static str = "&&";
    pub const OR: &'static str = "||";
    pub const EQ: &'static str = "==";
    pub const NEQ: &'static str = "!=";
    pub const LT: &'static str = "<";
    pub const GT: &'static str = ">";
    pub const LE: &'static str = "<=";
    pub const GE: &'static str = ">=";
    pub const IMPLIES: &'static str = "==>";
    pub const RANGE: &'static str = "..";

    pub fn symbol(&self) -> &'static str {
        use BinOp_ as B;
        match self {
            B::Add => B::ADD,
            B::Sub => B::SUB,
            B::Mul => B::MUL,
            B::Mod => B::MOD,
            B::Div => B::DIV,
            B::BitOr => B::BIT_OR,
            B::BitAnd => B::BIT_AND,
            B::Xor => B::XOR,
            B::Shl => B::SHL,
            B::Shr => B::SHR,
            B::And => B::AND,
            B::Or => B::OR,
            B::Eq => B::EQ,
            B::Neq => B::NEQ,
            B::Lt => B::LT,
            B::Gt => B::GT,
            B::Le => B::LE,
            B::Ge => B::GE,
            B::Implies => B::IMPLIES,
            B::Range => B::RANGE,
        }
    }

    pub fn is_pure(&self) -> bool {
        use BinOp_ as B;
        match self {
            B::Add | B::Sub | B::Mul | B::Mod | B::Div | B::Shl | B::Shr => false,
            B::BitOr
            | B::BitAnd
            | B::Xor
            | B::And
            | B::Or
            | B::Eq
            | B::Neq
            | B::Lt
            | B::Gt
            | B::Le
            | B::Ge
            | B::Range
            | B::Implies => true,
        }
    }

    pub fn is_spec_only(&self) -> bool {
        use BinOp_ as B;
        matches!(self, B::Range | B::Implies)
    }
}

impl FunctionVisibility {
    pub const PUBLIC: &'static str = "public";
    pub const SCRIPT: &'static str = "public(script)";
    pub const FRIEND: &'static str = "public(friend)";
    pub const INTERNAL: &'static str = "";
}

//**************************************************************************************************
// Display
//**************************************************************************************************

impl fmt::Display for ModuleIdent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}::{}", self.value.0, &self.value.1)
    }
}

impl fmt::Display for ModuleAccess_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ModuleAccess_::Name(n) => write!(f, "{}", n),
            ModuleAccess_::ModuleAccess(m, n) => write!(f, "{}::{}", m, n),
            ModuleAccess_::QualifiedModuleAccess(m, n) => write!(f, "{}::{}", m, n),
        }
    }
}

impl fmt::Display for UnaryOp_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

impl fmt::Display for BinOp_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

impl fmt::Display for FunctionVisibility {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                FunctionVisibility::Public(_) => FunctionVisibility::PUBLIC,
                FunctionVisibility::Script(_) => FunctionVisibility::SCRIPT,
                FunctionVisibility::Friend(_) => FunctionVisibility::FRIEND,
                FunctionVisibility::Internal => FunctionVisibility::INTERNAL,
            }
        )
    }
}

impl fmt::Display for Ability_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Ability_::Copy => Ability_::COPY,
                Ability_::Drop => Ability_::DROP,
                Ability_::Store => Ability_::STORE,
                Ability_::Key => Ability_::KEY,
            }
        )
    }
}
