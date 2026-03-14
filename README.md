<h1 align="center">MaxPanel</h1>

<p align="center">
  <b>Опенсорсная панель управления ботами в мессенджере Макс</b>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-BSL_1.1-yellow.svg" alt="License: BSL 1.1"></a>
  <a href="https://github.com/goduni/maxpanel/stargazers"><img src="https://img.shields.io/github/stars/goduni/maxpanel" alt="Stars"></a>
</p>

<p align="center">
  <a href="https://t.me/goduniblog">Telegram</a> &bull;
  <a href="https://t.me/+Cd5Rx2QPmytmZmYy">Чат</a> &bull;
  <a href="https://github.com/goduni">GitHub</a>
</p>

---

## Что это?

MaxPanel — self-hosted админ-панель для управления ботами в мессенджере [Макс](https://max.ru). Веб-интерфейс для просмотра диалогов, отправки сообщений, просмотра событий и вызова любых методов API — прямо из браузера.

В отличие от Telegram, Макс позволяет подключать несколько webhook/polling потребителей к одному боту, поэтому панель не ломает существующую инфраструктуру.

## Возможности

- **Веб и мобильная версия** — адаптивный дизайн, тёмная и светлая тема
- **Мультитенантная иерархия** — Организация > Проект > Бот с ролевой моделью (owner, admin, member, editor, viewer)
- **Просмотр диалогов** — пузырьки сообщений в стиле Telegram, рендеринг форматирования, медиа-прокси для видео, пересланные сообщения
- **Отправка сообщений** — текст с форматированием (жирный, курсив, зачёркнутый, подчёркнутый, код, ссылки), несколько файлов, инлайн-клавиатуры, стикеры, контакты, геолокация
- **История чатов** — синхронизация последних сообщений из Max API, подгрузка старых сообщений на лету
- **Лог событий** — все входящие/исходящие события с курсорной пагинацией и автообновлением
- **API-консоль** — вызов любого метода Max API прямо из браузера
- **Два режима получения событий** — webhook и long polling
- **Gateway API** — проксирование API-вызовов через сохранённые токены бота, пуш событий через M2M API
- **Безопасность** — AES-256-GCM шифрование токенов, JWT с ротацией refresh-токенов, rate limiting, CSP-заголовки

## Стек

| Слой | Технологии |
|------|-----------|
| **Backend** | Rust, Axum 0.8, SQLx 0.8, Tokio, PostgreSQL 16 |
| **Frontend** | React 19, TypeScript 5, Vite 6, Tailwind CSS 4, shadcn/ui, TanStack Query |
| **Инфра** | Docker Compose, nginx |

## Быстрый старт

```bash
# Клонируйте
git clone https://github.com/goduni/maxpanel.git
cd maxpanel

# Настройте переменные окружения
cp .env.example .env
# Отредактируйте .env — сгенерируйте секреты:
#   openssl rand -hex 32  (для BOT_TOKEN_ENCRYPTION_KEY и других секретов)

# Запустите
docker compose up --build
```

Фронтенд: `http://localhost:3000` | Backend API: `http://localhost:8080` | Swagger UI: `http://localhost:8080/swagger-ui`

## Архитектура

```
handlers   HTTP-слой: парсинг запросов, валидация, форматирование ответов
    |
services   Бизнес-логика, транзакции, вызовы Max API
    |
   db       Доступ к данным через SQL с проверкой на этапе компиляции (sqlx)
```

**Backend:**
- Компайл-тайм проверка SQL через SQLx офлайн-режим
- Партиционированная таблица событий (помесячные автопартиции)
- Фоновые воркеры: супервизор поллинга с backoff, менеджер партиций, очистка токенов
- Rate limiter (token bucket), middleware с security-заголовками
- OpenAPI-документация через utoipa + Swagger UI

**Frontend:**
- Feature-based структура: `features/{auth,organizations,projects,bots,events,chats,api-console}/`
- Трёхуровневая навигация: организация > проект > бот
- Infinite scroll с курсорной пагинацией
- i18n с русской локалью

## Переменные окружения

Полный список — в [`.env.example`](.env.example).

| Переменная | Описание |
|-----------|----------|
| `DATABASE_URL` | Строка подключения PostgreSQL |
| `JWT_SECRET` | HMAC-секрет для JWT (мин. 32 байта) |
| `BOT_TOKEN_ENCRYPTION_KEY` | 32-байтный hex-ключ AES-256 |
| `WEBHOOK_BASE_URL` | Публичный HTTPS URL для вебхуков |
| `APP_ENV` | `development` или `production` |

## Разработка

```bash
# Backend (нужен PostgreSQL)
cd backend
cargo sqlx migrate run
cargo run

# Frontend
cd frontend
npm install
npm run dev

# Тесты
cd backend
cargo test -- --test-threads=1

# Обновление SQL-кеша после изменения запросов
cd backend
cargo sqlx prepare
```

## Структура проекта

```
maxpanel/
  docker-compose.yml
  .env.example
  swagger.json            # Спецификация Max Bot API (справочник)
  backend/
    src/
      main.rs             # Точка входа, запуск воркеров, graceful shutdown
      config.rs           # Конфигурация из переменных окружения
      router.rs           # Определение маршрутов
      errors.rs           # Типизированные ошибки
      extractors/         # AuthUser, BotAuthContext
      handlers/           # HTTP-обработчики
      services/           # Бизнес-логика (crypto, ingestion, max_api, ...)
      db/                 # SQL-запросы (проверяются при компиляции)
      models/             # Row-типы (БД) и Response-типы (API)
      workers/            # Поллинг, партиции, очистка
      middleware/         # Rate limiter, security headers
    migrations/           # PostgreSQL-миграции
    .sqlx/                # Офлайн-кеш запросов
  frontend/
    src/
      app/                # Маршруты, провайдеры
      components/         # Лейаут, UI (shadcn)
      features/           # Auth, orgs, projects, bots, chats, events, console
      hooks/              # Infinite scroll, media queries
      lib/                # API-клиент, типы, утилиты
      stores/             # Zustand (auth, theme, sidebar)
```

## Contributing

Будем рады вашему вкладу! Ознакомьтесь с [CONTRIBUTING.md](CONTRIBUTING.md) перед созданием PR.

## Лицензия

[Business Source License 1.1](LICENSE)

Бесплатно для внутреннего использования. Перепродажа как коммерческий продукт или управляемый сервис требует отдельной лицензии. Переходит в Apache 2.0 через три года после каждого релиза.

---

<p align="center">
  by <a href="https://t.me/goduniblog">@goduniblog</a>
</p>
