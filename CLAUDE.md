# CLAUDE.md — правила проекта cyb-ts

## О проекте

**cyb** — веб-приложение для взаимодействия с блокчейном Bostrom (Cosmos-based).
Основные функции: поиск в knowledge graph (oracle), IPFS-контент, кошельки (Keplr), governance, стейкинг, транзакции, scripting (Rune), AI-интеграции.

## Рабочий процесс

- Для проверки после коммита — запускать `deno task build` (как CI), а не dev server
- Dev server (`deno task start`) запускать только по явному запросу пользователя
- Линтер: `deno task lint` (Biome). Исправление: `deno task lint:fix`
- Форматирование: `deno task format`

## Git

- **Push только по запросу** — никогда не пушить без явного указания пользователя
- **Атомарные коммиты** — один коммит на одну функцию/изменение, не смешивать несвязанные правки
- Ветки: `master` — продакшен, `dev` — стейджинг

## Планы и решения

Все утверждённые планы сохраняются в `.claude/plans/`, а не во временные папки.
Файл `reference/roadmap.md` — общий roadmap проекта.

## Стек

- Runtime: **Deno 2** (task runner, build, dev server, lint)
- Bundler: **Rspack 1.7** (Rust-based, webpack-совместимый API)
- Transpiler: **SWC** (через `builtin:swc-loader`, заменяет Babel)
- Linter/Formatter: **Biome** (Rust-based, заменяет ESLint + Prettier)
- Package manager: **Deno** (`deno install` для node_modules)
- Test runner: отсутствует (Jest удалён, тесты в src/ временно не запускаются)
- Framework: **React 18**, **TypeScript 5**
- State management: **Redux Toolkit** + **React Query** (TanStack Query v4)
- GraphQL: **Apollo Client** + codegen
- Routing: **React Router v6**
- Blockchain: **CosmJS** (@cosmjs/stargate, proto-signing, etc.)
- IPFS: **Helia**, **kubo-rpc-client**
- Styling: CSS + SCSS modules, PostCSS
- WASM: **cyb-cozo-lib-wasm** (CozoDb), **cyb-rune-wasm** (scripting)

## Сборка

```bash
# Установка зависимостей:
deno install

# Все команды через Deno:
deno task start               # dev server (HTTPS на localhost:3001, HMR)
deno task build               # production build → build/
deno task build-ipfs          # IPFS production build (PUBLIC_PATH=./)
deno task build-report        # build с профилированием → stats.json
deno task lint                # biome check src/
deno task lint:fix            # biome check --write src/
deno task format              # biome format --write src/
deno task serve               # serve production build (deno std/http)
deno task generate-graphql-types  # GraphQL codegen
```

Важно:
- `DENO_NO_PACKAGE_JSON=1` используется во всех deno tasks чтобы Deno не парсил package.json
- `package.json` содержит только зависимости и `generate-graphql-types` скрипт — все остальное в `deno.json`
- Node.js нужен только как runtime для Rspack (планируется убрать)
- Production build требует ~8GB памяти (`NODE_OPTIONS=--max-old-space-size=8192` в CI)

## CI/CD

Три GitHub Actions workflow (`.github/workflows/`):
- `build_cyb.yml` — push в `master` → GitHub Pages (ветка `build`)
- `build_cyb_rebyc.yml` — push в `dev` → GitHub Pages (ветка `rebyc-build`)
- `build_ipfs.yml` — push в `master` → IPFS cluster + Telegram уведомление

Все workflow используют Deno 2 + Node.js 22 (Node нужен для Rspack).

Деплой также возможен через Netlify и Docker.

## Структура проекта

```
src/
├── index.tsx              # Точка входа, provider tree (15+ nested providers)
├── router.tsx             # React Router v6 конфигурация
├── routes.ts              # Маршруты приложения
│
├── components/            # 72 переиспользуемых UI компонента
│   ├── atoms/             # Базовые атомарные компоненты
│   ├── account/           # Компоненты аккаунтов
│   ├── search/            # Поиск
│   ├── TextMarkdown/      # Рендеринг markdown
│   ├── contentIpfs/       # Отображение IPFS контента
│   └── ...
│
├── containers/            # 29 контейнеров (legacy, мигрировать в pages/features)
│   ├── application/       # Корневой layout приложения
│   ├── brain/             # Knowledge graph визуализация
│   ├── governance/        # Proposals
│   ├── oracle/            # Поисковый oracle
│   ├── warp/              # DEX/пулы ликвидности
│   └── ...
│
├── pages/                 # 11 страниц (новая структура)
│   ├── Brain/             # Oracle/brain page
│   ├── Hub/               # IBC Hub
│   ├── Keys/              # Key management
│   ├── oracle/            # Oracle search
│   ├── robot/             # Robot profile
│   ├── teleport/          # Teleport (swap/send/bridge)
│   ├── Settings/          # Settings
│   └── ...
│
├── features/              # 13 фич (feature-sliced подход)
│   ├── adviser/           # AI adviser (TypeIt анимация)
│   ├── cyberlinks/        # Cyberlink operations
│   ├── cybernet/          # Cybernet integration
│   ├── ipfs/              # IPFS features
│   ├── passport/          # User passport/identity
│   ├── particle/          # Particle (content unit)
│   ├── sense/             # Sense/notifications
│   ├── staking/           # Staking operations
│   ├── studio/            # Content studio
│   └── ...
│
├── services/              # 14 сервисов
│   ├── CozoDb/            # CozoDb (WASM, graph/relational queries)
│   ├── QueueManager/      # Task queue management
│   ├── backend/           # Backend channel service
│   ├── community/         # Community data
│   ├── graphql/           # Apollo Client setup
│   ├── ipfs/              # IPFS node management (Helia, kubo)
│   ├── lcd/               # LCD (REST) API queries
│   ├── neuron/            # Neuron operations
│   ├── passports/         # Passport resolution
│   ├── relayer/           # IBC relayer
│   ├── scripting/         # Rune scripting engine
│   ├── service-worker/    # Service worker (disabled)
│   ├── transactions/      # Transaction building/signing
│   └── soft.js            # Software management
│
├── redux/                 # Redux store
│   ├── store.ts           # configureStore (Redux Toolkit)
│   ├── hooks.ts           # Typed useSelector/useDispatch
│   ├── features/          # RTK slices: currentAccount, ibcDenom, pocket, warp
│   ├── reducers/          # Legacy reducers: backend, bandwidth, gol, scripting
│   └── actions/           # Legacy actions: bandwidth
│
├── contexts/              # 14 React context providers
│   ├── appData.tsx        # Application data (syncing, IPFS)
│   ├── backend/           # Backend connection
│   ├── chain/             # Chain configuration
│   ├── device.tsx         # Device detection
│   ├── hub.tsx            # IBC hub state
│   ├── ibcDenom.tsx       # IBC denomination mapping
│   ├── networks.tsx       # Network configuration
│   ├── queryClient.tsx    # CosmJS query client
│   ├── queryCyberClient.tsx # Cyber-specific query client
│   ├── scripting/         # Rune scripting context
│   ├── signerClient.tsx   # Transaction signing client (Keplr)
│   └── ...
│
├── hooks/                 # 30+ кастомных React хуков
│   ├── useCurrentAddress.ts     # Текущий адрес кошелька
│   ├── useGetMarketData.ts      # Рыночные данные
│   ├── useParticle.ts           # IPFS particle data
│   ├── useQueueIpfsContent.ts   # IPFS content queue
│   ├── useRuneMutation.ts       # Rune script execution
│   ├── useWaitForTransaction.ts # TX confirmation
│   ├── contract/                # Smart contract hooks
│   ├── governance/              # Governance hooks
│   ├── warp/                    # DEX/liquidity hooks
│   └── ...
│
├── utils/                 # Утилиты
│   ├── encoding.ts        # Web API replacements for Buffer (toHex, toBase64, toBytes)
│   ├── config.ts          # App configuration (chain, endpoints)
│   ├── address.ts         # Address utilities (bech32)
│   ├── keplrUtils.ts      # Keplr wallet utilities
│   ├── logging/           # cyblog global logger
│   ├── search/            # Search utilities
│   ├── ipfs/              # IPFS utilities
│   ├── chains/            # Chain configurations
│   ├── assert-shim.js     # Node assert polyfill
│   ├── os-shim.js         # Node os polyfill
│   └── ...
│
├── constants/             # Константы приложения
├── types/                 # TypeScript type definitions (globals, window, assets)
├── generated/             # Сгенерированные GraphQL типы (graphql.ts — ~15K строк)
├── websockets/            # WebSocket context и hooks
├── layouts/               # Layout компоненты (Main layout)
├── style/                 # Глобальные стили (CSS, SCSS, variables)
├── i18n/                  # Интернационализация (en.js)
├── hocs/                  # Higher-order components
├── image/                 # Статические изображения
├── sounds/                # Аудио файлы
├── stories/               # Storybook stories
└── fonts/                 # Шрифты
```

## Provider Tree (порядок вложенности)

Порядок провайдеров в `src/index.tsx` критичен — изменение порядка может сломать приложение:

```
Redux Provider → NetworksProvider → QueryClientProvider → SdkQueryClientProvider
→ CyberClientProvider → SigningClientProvider → HubProvider → IbcDenomProvider
→ WebsocketsProvider → DataProvider → ApolloProvider → BackendProvider
→ ScriptingProvider → DeviceProvider → AdviserProvider → ErrorBoundary
```

## Конфигурация Rspack

Три конфиг-файла:
- `rspack.config.common.js` — общий (loaders, plugins, resolve, aliases)
- `rspack.config.dev.js` — dev (HMR, HTTPS, incremental builds)
- `rspack.config.prod.js` — production (minification, code splitting, source maps)

Ключевые resolve aliases:
- `src` → `src/`
- `components` → `src/components`
- `images` → `src/image`
- `sounds` → `src/sounds`

Workers: `worker-rspack-loader` с кастомным `WorkerConfigPlugin` для проброса resolve конфига.

Файлы `.cozo` и `.rn` загружаются как source (asset/source).

## Biome (линтер/форматтер)

Конфиг: `biome.json`
- Indent: 2 spaces, LF line endings, max width 100
- Single quotes (JS), double quotes (JSX)
- Trailing commas: ES5
- `src/generated/**` исключён из проверки
- `noUnusedImports`: error — неиспользуемые импорты запрещены
- `noExplicitAny`: off — `any` разрешён
- A11y правила отключены

## TypeScript

- Target/Module: ES2020
- Strict mode: enabled
- JSX: `react-jsx` (automatic runtime)
- `noEmit: true` — компиляция через SWC, tsc только для проверки типов
- Type roots: `src/types/` + `node_modules/@types`
- `isolatedModules: true` — каждый файл транспилируется отдельно

## GraphQL

- Schema: `https://index.bostrom.cybernode.ai/v1/graphql`
- Файлы запросов: `src/**/*.graphql`
- Codegen output: `src/generated/graphql.ts`
- Генерация React hooks для Apollo Client

Запуск: `deno task generate-graphql-types`

## Безопасность

- Iframe: sandbox атрибут обязателен для любого IPFS/gateway контента
- CSP: Content-Security-Policy задана в `src/index.html`
- Scripting output: DOMPurify санитизация Rune script content_result в `src/services/scripting/services/postProcessing.ts`
- Secrets: хранятся в localStorage (unencrypted) — ключ `secrets`

## Node polyfills

- `@rspack/plugin-node-polyfill` удалён — был глобальный полифилл всех Node API
- `src/utils/encoding.ts` — замена Buffer.from() на Web APIs (toHex, toBase64, toBytes, fromBytes)
- `ProvidePlugin` точечно инжектит `process` и `Buffer` для сторонних пакетов
- `resolve.fallback` в rspack конфиге — полифиллы для сторонних пакетов (buffer, stream, events)
- Ряд Node API отключен (`fs`, `path`, `crypto`, `http`, etc. → `false`)
- LibP2P стабы: `@achingbrain/nat-port-mapper`, `default-gateway`, `execa` заменены пустыми модулями

## Ключевые паттерны

### State Management
- **Redux Toolkit** — глобальное состояние (currentAccount, ibcDenom, pocket, warp, scripting)
- **React Query (TanStack)** — серверное состояние, кэширование запросов
- **Apollo Client** — GraphQL запросы/подписки
- **React Context** — domain-specific state (14 контекстов)
- **localStorage** — персистентные настройки (pocket, settings)

### IPFS
- Helia node запускается в браузере
- kubo-rpc-client для внешнего IPFS API
- Content addressable через CID
- Particle = единица контента (текст, изображение, видео, etc.)

### Blockchain
- Cosmos SDK based (Bostrom chain)
- Keplr wallet integration для подписи транзакций
- CosmJS для взаимодействия с блокчейном (RPC, LCD, signing)
- IBC (Inter-Blockchain Communication) для кросс-чейн операций

### Scripting
- Rune WASM runtime для пользовательских скриптов
- Результаты санитизируются через DOMPurify
- Доступно через ScriptingProvider context

### Workers
- SharedWorker и Web Worker для тяжёлых операций
- ServiceWorker отключён (несовместим с Rspack)

## Environment Variables

Задаются через `DefinePlugin` и `docker-compose.yml`:
- `CHAIN_ID` — ID блокчейна (по умолчанию `bostrom`)
- `RPC_URL`, `LCD_URL`, `WEBSOCKET_URL` — endpoint'ы блокчейна
- `INDEX_HTTPS`, `INDEX_WEBSOCKET` — GraphQL indexer
- `CYBER_GATEWAY` — IPFS gateway URL
- `IPFS_DEPLOY` — флаг IPFS сборки (меняет public path на `./`)
- `COMMIT_SHA`, `BRANCH` — git info для отслеживания в build

## Статистика кодовой базы

- ~1050 TS/JS файлов (347 .ts, 518 .tsx, 52 .js, 117 .jsx)
- 72 компонента, 29 контейнеров, 11 страниц, 13 фич, 14 сервисов
- Production build: ~9.5s (Rspack), ранее ~100s (Webpack)
- ~117 JSX файлов ещё не конвертированы в TSX (legacy)
