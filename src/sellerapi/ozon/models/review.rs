use serde::{Deserialize, Serialize};

/// Ответ на запрос списка отзывов `/v1/review/list`
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewListResponse {
    /// Признак того, что есть ещё отзывы (true, если вернули не все отзывы)
    pub has_next: bool,

    /// Идентификатор последнего отзыва на странице
    pub last_id: String,

    /// Список отзывов
    pub reviews: Vec<Review>,
}

/// Отзыв покупателя
#[derive(Debug, Serialize, Deserialize)]
pub struct Review {
    /// Количество комментариев у отзыва
    pub comments_amount: i32,

    /// Идентификатор отзыва
    pub id: String,

    /// Признак участия в расчёте рейтинга
    pub is_rating_participant: bool,

    /// Статус заказа, на который оставлен отзыв:
    /// DELIVERED — доставлен,
    /// CANCELLED — отменён
    pub order_status: String,

    /// Количество изображений в отзыве
    pub photos_amount: i32,

    /// Дата публикации отзыва (формат RFC 3339, например 2019-08-24T14:15:22Z)
    pub published_at: String,

    /// Оценка отзыва
    pub rating: i32,

    /// Идентификатор товара (SKU в системе Ozon)
    pub sku: i64,

    /// Статус отзыва:
    /// UNPROCESSED — не обработан,
    /// PROCESSED — обработан
    pub status: String,

    /// Текст отзыва
    pub text: String,

    /// Количество видео в отзыве
    pub videos_amount: i32,
}

/// Ответ на запрос информации об отзыве `/v1/review/info`
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewInfoResponse {
    /// Количество комментариев к отзыву
    pub comments_amount: i32,

    /// Количество дизлайков на отзыве
    pub dislikes_amount: i32,

    /// Идентификатор отзыва
    pub id: String,

    /// Признак участия в расчёте рейтинга
    pub is_rating_participant: bool,

    /// Количество лайков на отзыве
    pub likes_amount: i32,

    /// Статус заказа, на который оставлен отзыв:
    /// DELIVERED — доставлен,
    /// CANCELLED — отменён
    pub order_status: String,

    /// Информация об изображениях, прикреплённых к отзыву
    pub photos: Vec<ReviewPhoto>,

    /// Количество изображений в отзыве
    pub photos_amount: i32,

    /// Дата публикации отзыва (формат RFC 3339, например 2019-08-24T14:15:22Z)
    pub published_at: String,

    /// Оценка отзыва
    pub rating: i32,

    /// Идентификатор товара (SKU в системе Ozon)
    pub sku: i64,

    /// Статус отзыва:
    /// UNPROCESSED — не обработан,
    /// PROCESSED — обработан
    pub status: String,

    /// Текст отзыва
    pub text: String,

    /// Информация о видео, прикреплённых к отзыву
    pub videos: Vec<ReviewVideo>,

    /// Количество видео в отзыве
    pub videos_amount: i32,
}

/// Информация об изображении в отзыве
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewPhoto {
    /// Высота изображения
    pub height: i32,

    /// Ссылка на изображение
    pub url: String,

    /// Ширина изображения
    pub width: i32,
}

/// Информация о видео в отзыве
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewVideo {
    /// Высота видео
    pub height: i64,

    /// Ссылка на превью видео
    pub preview_url: String,

    /// Ссылка на короткое превью видео
    pub short_video_preview_url: String,

    /// Ссылка на видео
    pub url: String,

    /// Ширина видео
    pub width: i64,
}

/// Ответ на запрос списка комментариев к отзыву `/v1/review/comment/list`
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewCommentListResponse {
    /// Список комментариев к отзыву
    pub comments: Vec<ReviewComment>,

    /// Количество элементов в выдаче
    pub offset: i32,
}

/// Комментарий к отзыву
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewComment {
    /// Идентификатор комментария
    pub id: String,

    /// Признак, что комментарий оставлен официальным лицом
    pub is_official: bool,

    /// Признак, что комментарий оставил продавец (true) или покупатель (false)
    pub is_owner: bool,

    /// Идентификатор родительского комментария (если это ответ на другой комментарий)
    pub parent_comment_id: String,

    /// Дата публикации комментария (формат RFC 3339, например 2019-08-24T14:15:22Z)
    pub published_at: String,

    /// Текст комментария
    pub text: String,
}

/// Ответ на создание комментария `/v1/review/comment/create`
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewCommentCreateResponse {
    /// Идентификатор созданного комментария
    pub comment_id: String,
}

/// Ответ на запрос количества отзывов по статусам `/v1/review/count`
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewsCountResponse {
    /// Количество обработанных отзывов
    pub processed: u32,

    /// Количество всех отзывов
    pub total: u32,

    /// Количество необработанных отзывов
    pub unprocessed: u32,
}
