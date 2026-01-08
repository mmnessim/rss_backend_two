#[cfg(all(feature = "sqlite", feature = "postgres"))]
compile_error!(r#"Features 'sqlite' and 'postgres' are mutually exclusive.
    To use postgres, pass build arguments --no-default-features --features postgres
    Sqlite is the default feature and database
"#);

#[cfg(not(any(feature = "sqlite", feature = "postgres")))]
compile_error!("Must enable either 'sqlite' or 'postgres' feature");