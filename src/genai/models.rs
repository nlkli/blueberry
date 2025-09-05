use serde::{Deserialize, Serialize};

pub const SYSTEM_ROLE: &str = "system";
pub const USER_ROLE: &str = "system";
pub const ASSISTANT_ROLE: &str = "system";

/// Сообщение в диалоге с LLM
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Message {
    /// Роль отправителя:
    /// - "system" — инструкции для модели
    /// - "user" — сообщение пользователя
    /// - "assistant" — ответ модели
    pub role: String,

    /// Текстовое содержимое сообщения
    pub content: String,
}

impl Message {
    pub fn new_user_msg<T: Into<String>>(content: T) -> Self {
        Self {
            role: USER_ROLE.to_string(),
            content: content.into(),
        }
    }
}

/// Запрос к модели генерации текста/чата
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ChatRequest {
    /// Имя модели (например: "gpt-4o", "claude-3-opus", "gemini-1.5-pro")
    pub model: String,

    /// Последовательность сообщений (контекст диалога)
    pub messages: Vec<Message>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Параметр "temperature":
    /// управляет креативностью/случайностью генерации
    /// (0.0 = детерминированный ответ, 1.0 = больше разнообразия)
    pub temperature: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Максимальное количество токенов в ответе
    pub max_tokens: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Включить ли потоковую передачу (streaming)
    /// true = ответ приходит частями, false = цельный ответ
    pub stream: Option<bool>,
}

/// Один вариант ответа модели
#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    /// Индекс варианта (обычно 0, если один ответ)
    pub index: u32,

    /// Сообщение от модели
    pub message: Message,

    /// Причина завершения генерации:
    /// - "stop" — нормальное завершение
    /// - "length" — достигнут лимит токенов
    /// - "tool_calls" — вызван инструмент (если поддерживается)
    pub finish_reason: Option<String>,
}

/// Статистика использования токенов
#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    /// Количество токенов в запросе
    pub prompt_tokens: u32,

    /// Количество токенов в ответе
    pub completion_tokens: u32,

    /// Общее количество токенов (prompt + completion)
    pub total_tokens: u32,
}

/// Стандартный ответ от API генеративной модели (OpenAI-style)
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Уникальный идентификатор запроса
    pub id: String,

    /// Тип объекта (например, "chat.completion")
    pub object: String,

    /// Время создания ответа (UNIX timestamp)
    pub created: u64,

    /// Использованная модель (например, "gpt-4o")
    pub model: String,

    /// Список сгенерированных вариантов ответа
    pub choices: Vec<Choice>,

    /// Статистика использования токенов
    pub usage: Option<Usage>,
}

impl ChatResponse {
    pub fn take_message(&mut self, index: usize) -> Option<Message> {
        self.choices
            .iter_mut()
            .find(|c| c.index == index as u32)
            .map(|c| std::mem::take(&mut c.message))
    }
}
