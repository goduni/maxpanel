# Содействие / Contributing

Спасибо за интерес к MaxPanel! Мы рады любому вкладу — от исправления багов до новых фич.

## Как помочь

1. **Баг-репорты** — создайте [Issue](https://github.com/goduni/maxpanel/issues) с описанием проблемы
2. **Фичи** — обсудите идею в [Issue](https://github.com/goduni/maxpanel/issues) перед началом работы
3. **Pull Request** — форкните репо, создайте ветку, отправьте PR

## Процесс разработки

```bash
# Форк и клон
git clone https://github.com/<ваш-username>/maxpanel.git
cd maxpanel

# Создайте ветку
git checkout -b feat/my-feature

# Внесите изменения, проверьте
cd backend && SQLX_OFFLINE=true cargo check
cd frontend && npm run build

# Коммит и PR
git commit -m "feat: описание изменения"
git push origin feat/my-feature
```

## Правила

- **Backend:** SQL-запросы через `sqlx::query!` / `sqlx::query_as!` (компайл-тайм проверка). После изменения запросов — `cargo sqlx prepare`.
- **Frontend:** TypeScript strict mode, без `any`. Компоненты — функциональные.
- **Коммиты:** формат `feat:`, `fix:`, `chore:`, `docs:`.
- **Тесты:** `cargo test -- --test-threads=1` должны проходить.
- **Без лишнего:** не добавляйте зависимости без обоснования.

## Код-стайл

- Rust: `cargo fmt` + `cargo clippy`
- TypeScript: ESLint конфиг проекта
- Следуйте существующим паттернам (handlers > services > db)

## Вопросы?

Задавайте в [Telegram-чате](https://t.me/+Cd5Rx2QPmytmZmYy) или в Issues.
