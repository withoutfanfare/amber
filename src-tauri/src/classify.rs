/// Classified connection error with human-readable message and remediation hint.
pub struct ClassifiedError {
    pub kind: &'static str,
    pub message: String,
    pub remediation: String,
}

/// Classify a database connection error from stderr output into a structured
/// error with a human-readable message and actionable remediation hint.
pub fn classify_connection_error(db_type: &str, stderr: &str) -> ClassifiedError {
    let lower = stderr.to_lowercase();

    // Authentication failures
    if matches_auth_failure(&lower) {
        let (user_hint, pass_hint) = match db_type {
            "mysql" => ("MySQL", "Check the username and password in your profile. For local MySQL, the default user is usually 'root'."),
            "postgresql" => ("PostgreSQL", "Check the username and password in your profile. For local PostgreSQL, try the 'postgres' superuser."),
            _ => ("database", "Verify that the username and password in your profile are correct."),
        };
        return ClassifiedError {
            kind: "authentication_failure",
            message: format!("{user_hint} authentication failed — invalid credentials."),
            remediation: pass_hint.to_string(),
        };
    }

    // Connection refused (server not running or wrong port)
    if matches_connection_refused(&lower) {
        let port_hint = match db_type {
            "mysql" => "Check that MySQL is running on the configured port (default 3306). Try: brew services start mysql",
            "postgresql" => "Check that PostgreSQL is running on the configured port (default 5432). Try: brew services start postgresql",
            _ => "Check that the database server is running and the port is correct.",
        };
        return ClassifiedError {
            kind: "connection_refused",
            message: "Connection refused — the database server is not accepting connections."
                .to_string(),
            remediation: port_hint.to_string(),
        };
    }

    // Timeout
    if matches_timeout(&lower) {
        return ClassifiedError {
            kind: "timeout",
            message: "Connection timed out — the server did not respond.".to_string(),
            remediation: "Check that the host address is correct and that no firewall is blocking the connection. If connecting via SSH tunnel, verify the tunnel host is reachable.".to_string(),
        };
    }

    // Unknown host / DNS resolution failure
    if matches_unknown_host(&lower) {
        return ClassifiedError {
            kind: "unknown_host",
            message: "Host not found — unable to resolve the server address.".to_string(),
            remediation: "Verify the host address in your profile. For local databases, use '127.0.0.1' or 'localhost'.".to_string(),
        };
    }

    // Permission denied (user can connect but lacks database access)
    if matches_permission_denied(&lower) {
        return ClassifiedError {
            kind: "permission_denied",
            message: "Permission denied — the user does not have access to this database.".to_string(),
            remediation: "Check that the configured user has the required permissions on the target database. You may need to GRANT access.".to_string(),
        };
    }

    // Database not found
    if matches_database_not_found(&lower, db_type) {
        return ClassifiedError {
            kind: "database_not_found",
            message: "Database not found — the specified database does not exist on this server.".to_string(),
            remediation: "Check the database name in your profile. You may need to create the database first.".to_string(),
        };
    }

    // Fallback: unclassified error
    ClassifiedError {
        kind: "unknown",
        message: format!("Connection failed: {stderr}"),
        remediation: "Review the error message above. Check that the server is running, credentials are correct, and the host/port are reachable.".to_string(),
    }
}

fn matches_auth_failure(lower: &str) -> bool {
    lower.contains("access denied")
        || lower.contains("authentication failed")
        || lower.contains("password authentication failed")
        || lower.contains("login failed")
        || lower.contains("invalid password")
        || lower.contains("no password supplied")
        || lower.contains("no pg_hba.conf entry")
}

fn matches_connection_refused(lower: &str) -> bool {
    lower.contains("connection refused")
        || lower.contains("can't connect to")
        || lower.contains("could not connect to server")
        || lower.contains("is the server running")
        || lower.contains("actively refused")
}

fn matches_timeout(lower: &str) -> bool {
    lower.contains("timed out")
        || lower.contains("timeout expired")
        || lower.contains("connection timeout")
        || lower.contains("could not receive data")
}

fn matches_unknown_host(lower: &str) -> bool {
    lower.contains("unknown host")
        || lower.contains("name or service not known")
        || lower.contains("nodename nor servname provided")
        || lower.contains("could not translate host name")
        || lower.contains("getaddrinfo failed")
        || lower.contains("host not found")
}

fn matches_permission_denied(lower: &str) -> bool {
    lower.contains("permission denied")
        || lower.contains("insufficient privilege")
        || lower.contains("access denied for user") && lower.contains("to database")
}

fn matches_database_not_found(lower: &str, db_type: &str) -> bool {
    match db_type {
        "mysql" => lower.contains("unknown database"),
        "postgresql" => lower.contains("does not exist") && lower.contains("database"),
        "sqlite" => lower.contains("unable to open database") || lower.contains("no such file"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_mysql_auth_failure() {
        let err =
            "ERROR 1045 (28000): Access denied for user 'root'@'localhost' (using password: YES)";
        let result = classify_connection_error("mysql", err);
        assert_eq!(result.kind, "authentication_failure");
        assert!(result.message.contains("authentication failed"));
        assert!(result.remediation.contains("username and password"));
    }

    #[test]
    fn classifies_pg_auth_failure() {
        let err = "psql: error: connection to server at \"127.0.0.1\", port 5432 failed: FATAL:  password authentication failed for user \"postgres\"";
        let result = classify_connection_error("postgresql", err);
        assert_eq!(result.kind, "authentication_failure");
    }

    #[test]
    fn classifies_mysql_connection_refused() {
        let err = "ERROR 2002 (HY000): Can't connect to local MySQL server through socket '/tmp/mysql.sock' (2)";
        let result = classify_connection_error("mysql", err);
        assert_eq!(result.kind, "connection_refused");
        assert!(result.remediation.contains("brew services start mysql"));
    }

    #[test]
    fn classifies_pg_connection_refused() {
        let err = "psql: error: could not connect to server: Connection refused\n\tIs the server running on host \"127.0.0.1\" and accepting TCP/IP connections on port 5432?";
        let result = classify_connection_error("postgresql", err);
        assert_eq!(result.kind, "connection_refused");
        assert!(result
            .remediation
            .contains("brew services start postgresql"));
    }

    #[test]
    fn classifies_timeout() {
        let err = "psql: error: connection timed out";
        let result = classify_connection_error("postgresql", err);
        assert_eq!(result.kind, "timeout");
        assert!(result.remediation.contains("firewall"));
    }

    #[test]
    fn classifies_unknown_host() {
        let err = "psql: error: could not translate host name \"db.invalid.local\" to address: nodename nor servname provided, or not known";
        let result = classify_connection_error("postgresql", err);
        assert_eq!(result.kind, "unknown_host");
        assert!(result.remediation.contains("127.0.0.1"));
    }

    #[test]
    fn classifies_permission_denied() {
        let err = "ERROR: permission denied for database mydb";
        let result = classify_connection_error("mysql", err);
        assert_eq!(result.kind, "permission_denied");
        assert!(result.remediation.contains("GRANT"));
    }

    #[test]
    fn classifies_mysql_unknown_database() {
        let err = "ERROR 1049 (42000): Unknown database 'nonexistent_db'";
        let result = classify_connection_error("mysql", err);
        assert_eq!(result.kind, "database_not_found");
    }

    #[test]
    fn classifies_pg_database_not_found() {
        let err = "psql: error: FATAL:  database \"nonexistent\" does not exist";
        let result = classify_connection_error("postgresql", err);
        assert_eq!(result.kind, "database_not_found");
    }

    #[test]
    fn fallback_for_unrecognised_error() {
        let err = "some completely unknown error output";
        let result = classify_connection_error("mysql", err);
        assert_eq!(result.kind, "unknown");
        assert!(result
            .message
            .contains("some completely unknown error output"));
    }
}
