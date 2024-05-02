use crate::ns::*;

pub struct SemanticHost {
    pub(crate) arena: ThingyArena,

    project_path: Option<String>,
    env_cache: RefCell<Option<Rc<HashMap<String, String>>>>,

    pub(crate) explicit_namespaces: RefCell<HashMap<String, Namespace>>,
    pub(crate) user_namespaces: RefCell<HashMap<String, Namespace>>,
    pub(crate) qnames: RefCell<HashMap<Namespace, HashMap<String, QName>>>,
    invalidation_thingy: Thingy,
    unresolved_thingy: Thingy,
    pub(crate) top_level_package: Package,
    void_type: Type,
    number_type: RefCell<Option<Thingy>>,
    int_type: RefCell<Option<Thingy>>,
    uint_type: RefCell<Option<Thingy>>,
    float_type: RefCell<Option<Thingy>>,
}

impl SemanticHost {
    pub fn new(options: SemanticHostOptions) -> Self {
        let arena = ThingyArena::new();
        let explicit_namespaces = RefCell::new(HashMap::new());
        let user_namespaces = RefCell::new(HashMap::new());
        let qnames = RefCell::new(HashMap::new());
        let void_type: Type = VoidType::new(&arena).into();
        let invalidation_thingy: Thingy = InvalidationThingy::new(&arena).into();
        let unresolved_thingy: Thingy = UnresolvedThingy::new(&arena).into();
        let top_level_package = Package::new(&arena, "".into());
        let host = Self {
            arena,
            project_path: options.project_path.clone(),
            env_cache: RefCell::new(None),
            explicit_namespaces,
            user_namespaces,
            qnames,
            top_level_package: top_level_package.clone(),
            invalidation_thingy,
            unresolved_thingy,
            void_type,
            number_type: RefCell::new(None),
            int_type: RefCell::new(None),
            uint_type: RefCell::new(None),
            float_type: RefCell::new(None),
        };

        // Initialize top level namespaces
        top_level_package.set_public_ns(Some(host.factory().create_public_namespace(Some(top_level_package.clone().into()))));
        top_level_package.set_internal_ns(Some(host.factory().create_internal_namespace(Some(top_level_package.clone().into()))));

        host
    }

    #[inline(always)]
    pub fn factory(&self) -> ThingyFactory {
        ThingyFactory(self)
    }

    pub fn top_level_package(&self) -> Package {
        self.top_level_package.clone()
    }

    pub fn invalidation_thingy(&self) -> Thingy {
        self.invalidation_thingy.clone()
    }

    pub fn unresolved_thingy(&self) -> Thingy {
        self.unresolved_thingy.clone()
    }

    pub fn void_type(&self) -> Type {
        self.void_type.clone()
    }

    global_lookup!(number_type, "Number");
    global_lookup!(int_type, "int");
    global_lookup!(uint_type, "uint");
    global_lookup!(float_type, "float");

    /// Preload environment variables from the main project's `.env` file
    /// using the DotEnv file format.
    pub fn env(&self) -> Rc<HashMap<String, String>> {
        if let Some(env) = self.env_cache.borrow().as_ref() {
            return env.clone();
        }
        let mut r = HashMap::<String, String>::new();
        if let Some(project_path) = self.project_path.as_ref() {
            if let Ok(iterator) = dotenvy::from_path_iter(project_path) {
                for item in iterator {
                    if let Ok((key, value)) = item {
                        r.insert(key, value);
                    }
                }
            }
        }
        let r = Rc::new(r);
        self.env_cache.replace(Some(r.clone()));
        r
    }
}

#[derive(Clone)]
pub struct SemanticHostOptions {
    /// The directory path of the main project being compiled,
    /// used for the `import.meta.env.EXAMPLE` accessors.
    pub project_path: Option<String>,
}

impl Default for SemanticHostOptions {
    fn default() -> Self {
        Self {
            project_path: None,
        }
    }
}

macro global_lookup {
    ($field:ident, $as3name:expr) => {
        /// Possibly unresolved.
        pub fn $field(&self) -> Thingy {
            if let Some(r) = self.$field.borrow().as_ref() {
                return r.clone();
            }
            if let Some(r) = self.top_level_package.properties(self).get(&self.factory().create_qname(&self.top_level_package.public_ns().unwrap(), $as3name.to_owned())) {
                self.$field.replace(Some(r.clone()));
                r
            } else {
                self.unresolved_thingy()
            }
        }
    },
}