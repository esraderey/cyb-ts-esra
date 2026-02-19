# CLAUDE.md — правила проекта cyb-ts

## Планы и решения

Все утверждённые планы сохраняются в `.claude/plans/`, а не во временные папки.
Файл `reference/roadmap.md` — общий roadmap проекта.

## Стек (текущий -> целевой)

- Runtime: Node.js 18 -> Deno 2
- Bundler: Webpack 5 -> Rspack (потом возможно Vite)
- Package manager: Yarn 1 -> Deno (npm-совместимый)
- Test runner: Jest -> Deno test
- Framework: React 18, TypeScript 5 (остаётся)

## Сборка

```bash
yarn start        # dev server
yarn build        # production build
yarn lint         # eslint
yarn test         # jest
```

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

## Известные проблемы

- TS ошибка: `reflect-metadata` в tsconfig.json types (не установлен пакет)
- `text-transform: lowercase` убран из global.css (было на всех элементах)
