## Specs

An command line tool to download audio books from the Fonos app.

### Features

- List all books in the user's library
- Download audiobook chapters
- Download book cover artwork
- Generate chapter-specific artwork with chapter numbers overlaid
- Embed artwork into individual chapter files
- Compile chapters into a single m4b audiobook file with embedded metadata, chapter markers, and cover artwork

#### List all books in the user's library

API: `https://production.fonos.dev/users/my-library`
with headers:
- Authorization: Bearer <JWT token>

Sample response:
```json
[
  {
    "id": 1103857, // Internal book ID
    "entityId": 35,
    "entity": "english_book",
    "purchasedType": "credit",
    "purchasedAt": "2024-01-15T16:52:30.304Z"
  },
]
```
`entityId` is the external book ID.

#### Get book details

API: `https://production.fonos.dev/books/{entityId}`
with headers:
- Authorization: Bearer <JWT token>

Sample response:
```json
{
    "id": 1120,
    "title": "Pháo Đài Số",
    "description": "Cơ quan An ninh Quốc gia NSA đảm nhiệm trọng trách bảo vệ các liên lạc của chính phủ Hoa Kỳ và thu thập liên lạc của các quốc gia khác. Và con át chủ bài của họ chính là TRANSLTR, cỗ máy vạn năng có thể bẻ mọi mật mã. Cho đến khi nó đụng độ Pháo Đài Số bí ẩn - thuật toán xoay vòng văn bản sạch có thể tạo ra những mật mã không thể giải mã và khiến TRANSLTR trở nên lỗi thời.\n\nSusan Fletcher, trưởng Ban Mật mã của NSA, được triệu tập gấp để giải quyết khủng hoảng. Cô không ngờ rằng mình sẽ buộc phải chiến đấu, không chỉ cho niềm tin và an nguy của đất nước, mà còn cho tính mạng của chính mình cùng người đàn ông mà cô yêu.\n\nTất cả đều nỗ lực đến tuyệt vọng để phá hủy một sáng tạo không thể tưởng tượng nổi của một thiên tài tật nguyền... một thuật toán sẽ xóa sổ toàn bộ ngành tình báo và đe doạ cán cân quyền lực của chính phủ Hoa Kỳ. Mãi mãi.",
    "duration": 58643.80322000004,
    "commission": 0,
    "foreignCommission": null,
    "coverImageUrl": "books/0c2447e0-da05-11ee-a680-314604d714e3/coverImage/1709542570846.png",
    "datePublished": "2024-03-25T22:31:45.919Z",
    "fileSampleUrl": "books/0c2447e0-da05-11ee-a680-314604d714e3/sample/1710903312194.m4a",
    "fileSampleDuration": 742.388367,
    "price": 299000,
    "discountType": null,
    "discount": null,
    "status": "Published",
    "storageKey": "books/0c2447e0-da05-11ee-a680-314604d714e3",
    "coverColor": "#348c74",
    "productId": "vn.fonos.mobile.book.1120",
    "bookContentReview": 0,
    "voiceReview": 0,
    "digitalPublisher": "NHÀ XUÁT BẢN THÔNG TIN VÀ TRUYỀN THÔNG",
    "paperPublisher": "NXB LAO ĐỘNG",
    "responsiblePublisher": null,
    "editor": null,
    "responsibleContent": "Giám đốc - Tổng biên tập Trần Chí Đạt",
    "manuscriptCurator": null,
    "editorAndPrintEditing": "Nguyễn Tiến Phát, Phạm Thị Thanh",
    "publishDecisionNumber": "312/QĐ - NXB TTTT ngày 18 tháng 09 năm 2024",
    "publishConfirmationNumber": "2637-2024/CXBIPH/4-91/TTTT",
    "isbnNumber": "978-604-80-9840-7",
    "depositary": "Quý III năm 2024",
    "slug": "phao-dai-so-1709542572964",
    "isFree": false,
    "membershipIncluded": false,
    "whyItFree": null,
    "isHideRevenue": true,
    "isLibIncluded": false,
    "includedLibCommission": null,
    "platformFeePercentage": null,
    "taxFeePercentage": null,
    "fileSamples": [
        {
            "url": "books/0c2447e0-da05-11ee-a680-314604d714e3/sample/1710903312194.m4a",
            "voiceId": null,
            "duration": 742.388367
        }
    ],
    "isAiBook": false,
    "multiVoices": false,
    "createdBy": 67589,
    "updatedBy": 468127,
    "authorId": null,
    "publisherId": 295233,
    "foreignPublisherId": null,
    "categories": [
        {
            "id": 44,
            "name": "Văn học",
            "showOnList": true,
            "showOnDetail": true,
            "books_categories": {
                "primary": true
            }
        },
        {
            "id": 177,
            "name": "Tiểu thuyết Giả tưởng",
            "showOnList": true,
            "showOnDetail": true,
            "books_categories": {
                "primary": false
            }
        }
    ],
    "collections": [],
    "chapters": [
        {
            "id": 31081,
            "name": "CHƯƠNG 1",
            "duration": 662.418844,
            "order": 1,
            "url": "books/0c2447e0-da05-11ee-a680-314604d714e3/chapters/1710902210902.m4a",
            "voices": [
                {
                    "url": "books/0c2447e0-da05-11ee-a680-314604d714e3/chapters/1710902210902.m4a",
                    "voiceId": null
                }
            ],
            "attachments": [
                {
                    "url": "books/0c2447e0-da05-11ee-a680-314604d714e3/attachments/1710411792608.pdf",
                    "name": "Chương 1 - Tài liệu đính kèm"
                }
            ]
        },
    ]
}
```

`chapters.url` is the URL of the chapter audio file.

#### Download chapter audio file

API: `https://production.fonos.dev/books/{entityId}/chapters/{chapterId}`
with headers:
- Authorization: Bearer <JWT token>

Sample response:
```json
{
  "url": "https://production.fonos.dev/books/{entityId}/chapters/{chapterId}"
}
```

#### Get the resource permissions

API: `https://production.fonos.dev/resource-permissions?entity=books&id={bookId}&skipCached=true`
with headers:
- Authorization: Bearer <JWT token>

Sample response:
```json
{
  "cloud-cdn-signed-cookie": "cloud-cdn-signed-cookie",
  "cloud-cdn-signed-url": "cloud-cdn-signed-url",
  "azure-token-auth": "azure-token-auth",
  "vnetwork-token-auth": "vnetwork-token-auth",
  "byteplus-signed-token": "byteplus-signed-token",
  "clearKey": "clearKey"
}
```

#### Download chapter audio file from CDN

API: `https://cdn.fonos.dev/books/2aa4e1f0-439d-11ef-afc2-f533dbb1a713/chapters/1721150879158_64K.aac`
Headers:
- Cookie: above response's `cloud-cdn-signed-cookie`
- clearKey: above response's `clearKey`
- authType: "header"
- X-Playback-Session-Id: Unique session ID
- Range: For partial content requests (e.g., "bytes=0-11885644")

Sample request:
```
GET https://cdn.fonos.dev/books/2aa4e1f0-439d-11ef-afc2-f533dbb1a713/chapters/1721150879158_64K.aac
Headers:
- Cookie: above response's `cloud-cdn-signed-cookie`
- clearKey: above response's `clearKey`
- authType: "header"
- User-Agent: AppleCoreMedia/1.0.0.24G90 (Macintosh; U; Intel Mac OS X 18_6; en_us)

