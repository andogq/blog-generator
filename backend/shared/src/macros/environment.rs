#[macro_export]
macro_rules! get_from_environment {
    ($environment:ident, $var:expr) => {
        $environment
            .get($var)
            .ok_or(SourceError::MissingEnvVar($var.to_string()))?
            .to_string()
    };
}

#[macro_export]
macro_rules! init_from_environment {
    ($environment:ident {
        $($env_key:ident: $var:expr),*
    }, { $($key:ident: $value:expr),* }) => {
        Self {
            $(
                $env_key: get_from_environment!($environment, $var)
            ),*
            $(
                $key: $value
            ),*
        }
    };
    ($environment:ident {
        $($env_key:ident: $var:expr),*
    }) => {
        init_from_environment!($environment {
            $($env_key: $var),*
        },
        {})
    }
}
