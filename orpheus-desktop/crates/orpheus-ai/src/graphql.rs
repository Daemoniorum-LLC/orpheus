//! GraphQL client for Leviathan API
//!
//! Handles GraphQL queries and mutations to the Leviathan backend.

use serde::{Deserialize, Serialize};

/// GraphQL request wrapper
#[derive(Debug, Serialize)]
pub struct GraphQLRequest {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<String>,
}

impl GraphQLRequest {
    /// Create a new GraphQL request
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            variables: None,
            operation_name: None,
        }
    }

    /// Add variables
    pub fn with_variables(mut self, variables: serde_json::Value) -> Self {
        self.variables = Some(variables);
        self
    }

    /// Set operation name
    pub fn with_operation_name(mut self, name: impl Into<String>) -> Self {
        self.operation_name = Some(name.into());
        self
    }
}

/// GraphQL response wrapper
#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQLError>>,
}

impl<T> GraphQLResponse<T> {
    /// Check if the response has errors
    pub fn has_errors(&self) -> bool {
        self.errors.as_ref().map(|e| !e.is_empty()).unwrap_or(false)
    }

    /// Get first error message
    pub fn first_error(&self) -> Option<&str> {
        self.errors
            .as_ref()
            .and_then(|e| e.first())
            .map(|e| e.message.as_str())
    }
}

/// GraphQL error
#[derive(Debug, Deserialize)]
pub struct GraphQLError {
    pub message: String,
    #[serde(default)]
    pub locations: Vec<GraphQLErrorLocation>,
    #[serde(default)]
    pub path: Vec<serde_json::Value>,
    #[serde(default)]
    pub extensions: Option<serde_json::Value>,
}

/// GraphQL error location
#[derive(Debug, Deserialize)]
pub struct GraphQLErrorLocation {
    pub line: u32,
    pub column: u32,
}

// ============================================================================
// Conversation Session Types
// ============================================================================

/// Create conversation session input
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConversationSessionInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Create conversation session result
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConversationSessionResult {
    pub session: Option<ConversationSession>,
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
}

/// Conversation session
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationSession {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub project_path: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    pub created_at: String,
    pub last_activity: String,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub message_count: i32,
}

/// Create conversation message input
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConversationMessageInput {
    pub session_id: String,
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools_used: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_approval: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Create conversation message result
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConversationMessageResult {
    pub message: Option<ConversationMessage>,
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
}

/// Conversation message
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
    #[serde(default)]
    pub tools_used: Vec<String>,
    #[serde(default)]
    pub requires_approval: bool,
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub intent_type: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

// ============================================================================
// Agent Task Types
// ============================================================================

/// Create agent task input
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentTaskInput {
    pub persona_code: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    pub project_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_iterations: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_approval: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Create agent task result
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentTaskResult {
    pub task: Option<AgentTask>,
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
}

/// Agent task
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTask {
    pub task_id: String,
    pub persona_code: String,
    #[serde(default)]
    pub goal: Option<String>,
    pub description: String,
    pub project_path: String,
    pub status: String,
    pub max_iterations: i32,
    pub requires_approval: bool,
    pub submitted_at: String,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub result: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

// ============================================================================
// Health Check Types
// ============================================================================

/// Health status response
#[derive(Debug, Clone, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    #[serde(default)]
    pub timestamp: Option<i64>,
    #[serde(default)]
    pub checks: Option<HealthChecks>,
}

/// Health check components
#[derive(Debug, Clone, Deserialize)]
pub struct HealthChecks {
    #[serde(default)]
    pub database: Option<ComponentHealth>,
    #[serde(default)]
    pub redis: Option<ComponentHealth>,
    #[serde(default)]
    pub ai_providers: Option<ComponentHealth>,
    #[serde(default)]
    pub disk: Option<ComponentHealth>,
}

/// Component health status
#[derive(Debug, Clone, Deserialize)]
pub struct ComponentHealth {
    pub status: String,
}

impl HealthStatus {
    /// Check if the service is healthy
    pub fn is_healthy(&self) -> bool {
        self.status == "UP"
    }
}

// ============================================================================
// GraphQL Query Templates
// ============================================================================

/// GraphQL queries and mutations
pub mod queries {
    /// Create a conversation session
    pub const CREATE_CONVERSATION_SESSION: &str = r#"
        mutation CreateConversationSession($input: CreateConversationSessionInput!) {
            createConversationSession(input: $input) {
                success
                error
                session {
                    id
                    workspaceId
                    projectPath
                    userId
                    createdAt
                    lastActivity
                    messageCount
                    metadata
                }
            }
        }
    "#;

    /// Create a conversation message
    pub const CREATE_CONVERSATION_MESSAGE: &str = r#"
        mutation CreateConversationMessage($input: CreateConversationMessageInput!) {
            createConversationMessage(input: $input) {
                success
                error
                message {
                    id
                    sessionId
                    role
                    content
                    timestamp
                    toolsUsed
                    requiresApproval
                    taskId
                    intentType
                    metadata
                }
            }
        }
    "#;

    /// Get session messages
    pub const GET_SESSION_MESSAGES: &str = r#"
        query GetSessionMessages($sessionId: String!, $page: Int, $size: Int) {
            sessionMessages(sessionId: $sessionId, page: $page, size: $size) {
                edges {
                    node {
                        id
                        sessionId
                        role
                        content
                        timestamp
                        toolsUsed
                        requiresApproval
                    }
                }
                pageInfo {
                    hasNextPage
                    hasPreviousPage
                    totalCount
                }
            }
        }
    "#;

    /// Create an agent task
    pub const CREATE_AGENT_TASK: &str = r#"
        mutation CreateAgentTask($input: CreateAgentTaskInput!) {
            createAgentTask(input: $input) {
                success
                error
                task {
                    taskId
                    personaCode
                    goal
                    description
                    projectPath
                    status
                    maxIterations
                    requiresApproval
                    submittedAt
                    startedAt
                    completedAt
                    result
                    error
                    metadata
                }
            }
        }
    "#;

    /// Get agent task by ID
    pub const GET_AGENT_TASK: &str = r#"
        query GetAgentTask($taskId: ID!) {
            agentTask(taskId: $taskId) {
                taskId
                personaCode
                goal
                description
                projectPath
                status
                maxIterations
                requiresApproval
                submittedAt
                startedAt
                completedAt
                result
                error
                metadata
            }
        }
    "#;

    /// Get persona by code
    pub const GET_PERSONA: &str = r#"
        query GetPersona($code: String!) {
            persona(code: $code) {
                id
                code
                name
                description
                active
                defaultModel
                provider
                temperature
                topP
            }
        }
    "#;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphql_request_creation() {
        let req = GraphQLRequest::new("query { test }");
        assert_eq!(req.query, "query { test }");
        assert!(req.variables.is_none());
        assert!(req.operation_name.is_none());
    }

    #[test]
    fn test_graphql_request_with_variables() {
        let vars = serde_json::json!({ "id": "123" });
        let req = GraphQLRequest::new("query { test }")
            .with_variables(vars.clone())
            .with_operation_name("TestQuery");

        assert!(req.variables.is_some());
        assert_eq!(req.operation_name, Some("TestQuery".to_string()));
    }

    #[test]
    fn test_graphql_response_errors() {
        let response: GraphQLResponse<()> = GraphQLResponse {
            data: None,
            errors: Some(vec![GraphQLError {
                message: "Test error".to_string(),
                locations: vec![],
                path: vec![],
                extensions: None,
            }]),
        };

        assert!(response.has_errors());
        assert_eq!(response.first_error(), Some("Test error"));
    }

    #[test]
    fn test_health_status() {
        let status = HealthStatus {
            status: "UP".to_string(),
            timestamp: Some(1234567890),
            checks: None,
        };
        assert!(status.is_healthy());

        let status_down = HealthStatus {
            status: "DOWN".to_string(),
            timestamp: None,
            checks: None,
        };
        assert!(!status_down.is_healthy());
    }
}
