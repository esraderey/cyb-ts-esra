# CLAUDE.md — правила проекта cyb-ts

## Рабочий процесс

После каждого коммита — запустить dev server (`deno task start`) и показать результат на localhost.
Пользователь хочет видеть, что изменения работают, прежде чем двигаться дальше.

## Планы и решения

Все утверждённые планы сохраняются в `.claude/plans/`, а не во временные папки.
Файл `reference/roadmap.md` — общий roadmap проекта.

## Стек

- Runtime: **Deno 2** (task runner) + Node.js (для yarn install — пока есть GitHub deps в транзитивных зависимостях)
- Bundler: **Rspack 5.75** (Rust-based, drop-in замена Webpack)
- Package manager: **Yarn 1** (для установки npm deps) + **deno task** (для запуска скриптов)
- Test runner: Jest (будет заменён на Deno test — Фаза 4)
- Framework: React 18, TypeScript 5

## Сборка

```bash
# Предпочтительный способ (Deno):
deno task start        # dev server (HTTPS, HMR)
deno task build        # production build
deno task lint         # eslint

# Legacy (Yarn, тоже работает):
yarn start
yarn build
yarn lint

# Установка зависимостей (только через yarn, не deno install):
yarn install
```

Важно: `deno install` не работает из-за `aframe` -> `three-bmfont-text` GitHub dependency.
Используется `DENO_NO_PACKAGE_JSON=1` чтобы Deno не парсил package.json.

## Структура

- `src/components/` — переиспользуемые UI компоненты
- `src/containers/` — контейнеры страниц (legacy, мигрировать в pages/features)
- `src/pages/` — страницы
- `src/features/` — фичи (feature-sliced подход)
- `src/services/` — сервисы (IPFS, backend, scripting, etc.)
- `src/redux/` — Redux store
- `src/utils/` — утилиты
- `src/contexts/` — React context providers
- `src/hooks/` — кастомные хуки
- `src/generated/` — сгенерированные GraphQL типы

## Безопасность

- Iframe: sandbox атрибут обязателен для любого IPFS/gateway контента
- CSP: Content-Security-Policy задана в `src/index.html`
- Scripting output: DOMPurify санитизация Rune script content_result в `src/services/scripting/services/postProcessing.ts`
- Secrets: хранятся в localStorage (unencrypted) — ключ `secrets`
