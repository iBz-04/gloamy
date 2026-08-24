<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { Icon } from '@iconify/vue'
import { useAuthStore } from '@/stores/auth'

interface ConfigEntry {
  key: string
  value: string
  type: 'string' | 'number' | 'boolean' | 'array' | 'object'
  section: string
  rawLine: string
  lineIndex: number
}

interface ConfigSection {
  name: string
  icon: string
  description: string
  entries: ConfigEntry[]
}

interface SchemaField {
  key: string
  description: string
  type: 'string' | 'number' | 'boolean' | 'array' | 'object'
  default?: string
  example?: string
  options?: string[]
}

// Full schema knowledge base derived from config/schema.rs
const SCHEMA_FIELDS: Record<string, SchemaField[]> = {
  '_root': [
    { key: 'api_key', description: 'API key for the selected provider (e.g. OpenAI, Anthropic)', type: 'string', example: '"sk-..."' },
    { key: 'api_url', description: 'Base URL override for provider API (e.g. remote Ollama)', type: 'string', example: '"http://10.0.0.1:11434"' },
    { key: 'default_provider', description: 'Provider ID or alias', type: 'string', example: '"openai"', options: ['openai', 'anthropic', 'gemini', 'ollama', 'openrouter', 'glm', 'compatible'] },
    { key: 'default_model', description: 'Default model routed through the selected provider', type: 'string', example: '"gpt-4o"' },
    { key: 'default_temperature', description: 'Model temperature (0.0–2.0)', type: 'number', default: '0.7', example: '0.7' },
  ],
  'agent': [
    { key: 'compact_context', description: 'Use compact context for small models (13B or smaller)', type: 'boolean', default: 'false' },
    { key: 'max_tool_iterations', description: 'Max tool-call loop turns per user message', type: 'number', default: '50', example: '50' },
    { key: 'max_history_messages', description: 'Max conversation history messages retained per session', type: 'number', default: '50', example: '50' },
    { key: 'parallel_tools', description: 'Enable parallel tool execution within a single iteration', type: 'boolean', default: 'false' },
    { key: 'tool_dispatcher', description: 'Tool dispatch strategy', type: 'string', default: '"auto"', options: ['auto'] },
    { key: 'self_learning', description: 'Enable automatic self-learning from tool-call errors', type: 'boolean', default: 'true' },
    { key: 'max_lessons_per_query', description: 'Max lesson memories injected per turn', type: 'number', default: '3', example: '3' },
  ],
  'autonomy': [
    { key: 'level', description: 'Autonomy level controlling what the agent can do', type: 'string', default: '"full"', options: ['read_only', 'supervised', 'full'] },
    { key: 'workspace_only', description: 'Restrict filesystem access to workspace-relative paths', type: 'boolean', default: 'false' },
    { key: 'allowed_commands', description: 'Allowlist of executable names for shell execution', type: 'array', example: '["*"]' },
    { key: 'forbidden_paths', description: 'Explicit path denylist', type: 'array', example: '["/etc", "/sys"]' },
    { key: 'max_actions_per_hour', description: 'Max actions allowed per hour', type: 'number', default: '500' },
    { key: 'max_cost_per_day_cents', description: 'Max cost per day in cents', type: 'number', default: '5000' },
    { key: 'require_approval_for_medium_risk', description: 'Require explicit approval for medium-risk shell commands', type: 'boolean', default: 'true' },
    { key: 'block_high_risk_commands', description: 'Block high-risk shell commands even if allowlisted', type: 'boolean', default: 'true' },
    { key: 'shell_env_passthrough', description: 'Extra env vars allowed for shell tool subprocesses', type: 'array', example: '["MY_VAR"]' },
    { key: 'auto_approve', description: 'Tools that never require approval (e.g. read-only tools)', type: 'array', example: '["file_read"]' },
    { key: 'always_ask', description: 'Tools that always require interactive approval', type: 'array', example: '["shell"]' },
    { key: 'allowed_roots', description: 'Allowed filesystem roots for path resolution', type: 'array', example: '["~"]' },
  ],
  'observability': [
    { key: 'backend', description: 'Observability backend', type: 'string', default: '"none"', options: ['none', 'log', 'prometheus', 'otel'] },
    { key: 'otel_endpoint', description: 'OTLP endpoint (used when backend = "otel")', type: 'string', example: '"http://localhost:4318"' },
    { key: 'otel_service_name', description: 'Service name reported to OTel collector', type: 'string', default: '"gloamy"' },
    { key: 'runtime_trace_mode', description: 'Runtime trace storage mode', type: 'string', default: '"none"', options: ['none', 'rolling', 'full'] },
    { key: 'runtime_trace_path', description: 'Runtime trace file path', type: 'string', default: '"state/runtime-trace.jsonl"' },
    { key: 'runtime_trace_max_entries', description: 'Max entries retained in rolling mode', type: 'number', default: '200' },
  ],
  'security': [
    { key: 'pairing_key', description: 'Pre-shared pairing key for gateway auth', type: 'string', example: '"your-secret-key"' },
  ],
  'gateway': [
    { key: 'port', description: 'Gateway server port', type: 'number', default: '42617' },
    { key: 'host', description: 'Gateway server host', type: 'string', default: '"127.0.0.1"', example: '"0.0.0.0"' },
    { key: 'require_pairing', description: 'Require pairing before accepting requests', type: 'boolean', default: 'true' },
    { key: 'allow_public_bind', description: 'Allow binding to non-localhost without a tunnel', type: 'boolean', default: 'false' },
    { key: 'pair_rate_limit_per_minute', description: 'Max /pair requests per minute per client', type: 'number', default: '10' },
    { key: 'webhook_rate_limit_per_minute', description: 'Max /webhook requests per minute per client', type: 'number', default: '60' },
    { key: 'trust_forwarded_headers', description: 'Trust X-Forwarded-For headers (only behind trusted proxy)', type: 'boolean', default: 'false' },
    { key: 'idempotency_ttl_secs', description: 'TTL for webhook idempotency keys', type: 'number', default: '300' },
  ],
  'channels_config': [
    { key: 'cli', description: 'Enable CLI channel', type: 'boolean', default: 'true' },
    { key: 'message_timeout_secs', description: 'Max seconds per channel turn', type: 'number', default: '300', example: '1200' },
  ],
  'channels_config.telegram': [
    { key: 'bot_token', description: 'Telegram bot token from @BotFather', type: 'string', example: '"123456:your-token"' },
    { key: 'allowed_users', description: 'Allowed Telegram usernames or ["*"] for all', type: 'array', example: '["*"]' },
    { key: 'stream_mode', description: 'Message streaming mode', type: 'string', default: '"partial"', options: ['none', 'partial', 'full'] },
    { key: 'draft_update_interval_ms', description: 'Interval between streaming draft updates (ms)', type: 'number', default: '1000' },
    { key: 'interrupt_on_new_message', description: 'Interrupt current response when new message arrives', type: 'boolean', default: 'true' },
    { key: 'mention_only', description: 'Only respond to @mentions in groups', type: 'boolean', default: 'false' },
  ],
  'channels_config.discord': [
    { key: 'bot_token', description: 'Discord bot token', type: 'string', example: '"your-discord-bot-token"' },
    { key: 'allowed_users', description: 'Allowed Discord user IDs or ["*"]', type: 'array', example: '["*"]' },
    { key: 'guild_id', description: 'Restrict to a specific guild/server ID', type: 'string', example: '"123456789"' },
  ],
  'channels_config.slack': [
    { key: 'bot_token', description: 'Slack bot token (xoxb-...)', type: 'string', example: '"xoxb-your-token"' },
    { key: 'app_token', description: 'Slack app token for socket mode (xapp-...)', type: 'string', example: '"xapp-your-token"' },
    { key: 'allowed_users', description: 'Allowed Slack user IDs or ["*"]', type: 'array', example: '["*"]' },
  ],
  'memory': [
    { key: 'backend', description: 'Memory storage backend', type: 'string', default: '"sqlite"', options: ['sqlite', 'lucid', 'postgres', 'qdrant', 'markdown', 'none'] },
    { key: 'auto_save', description: 'Auto-save user conversation input to memory', type: 'boolean', default: 'true' },
    { key: 'hygiene_enabled', description: 'Run memory archiving and retention cleanup', type: 'boolean', default: 'true' },
    { key: 'archive_after_days', description: 'Archive session files older than N days', type: 'number', default: '7' },
    { key: 'purge_after_days', description: 'Purge archived files older than N days', type: 'number', default: '30' },
    { key: 'conversation_retention_days', description: 'Prune conversation rows older than N days (sqlite)', type: 'number', default: '30' },
    { key: 'embedding_provider', description: 'Embedding provider for semantic search', type: 'string', default: '"none"', options: ['none', 'openai', 'custom:URL'] },
    { key: 'embedding_model', description: 'Embedding model name', type: 'string', default: '"text-embedding-3-small"' },
    { key: 'embedding_dimensions', description: 'Embedding vector dimensions', type: 'number', default: '1536' },
    { key: 'vector_weight', description: 'Vector similarity weight in hybrid search (0.0–1.0)', type: 'number', default: '0.7' },
    { key: 'keyword_weight', description: 'BM25 keyword weight in hybrid search (0.0–1.0)', type: 'number', default: '0.3' },
    { key: 'min_relevance_score', description: 'Minimum score for memory inclusion in context', type: 'number', default: '0.4' },
    { key: 'response_cache_enabled', description: 'Cache LLM responses to avoid duplicate prompt costs', type: 'boolean', default: 'false' },
    { key: 'response_cache_ttl_minutes', description: 'TTL for cached responses in minutes', type: 'number', default: '60' },
    { key: 'snapshot_enabled', description: 'Enable periodic export of memories to MEMORY_SNAPSHOT.md', type: 'boolean', default: 'false' },
    { key: 'auto_hydrate', description: 'Auto-hydrate from MEMORY_SNAPSHOT.md when brain.db is missing', type: 'boolean', default: 'true' },
  ],
  'composio': [
    { key: 'enabled', description: 'Enable Composio integration for 1000+ OAuth tools', type: 'boolean', default: 'true' },
    { key: 'api_key', description: 'Composio API key — get one at composio.dev', type: 'string', example: '"your-composio-key"' },
    { key: 'entity_id', description: 'Default entity ID for multi-user setups', type: 'string', default: '"default"' },
  ],
  'one': [
    { key: 'enabled', description: 'Enable One CLI integration for 200+ platforms (Gmail, Slack, GitHub...)', type: 'boolean', default: 'true' },
    { key: 'api_key', description: 'One API key — get one at one.dev', type: 'string', example: '"your-one-key"' },
  ],
  'secrets': [
    { key: 'encrypt', description: 'Enable encryption for API keys and tokens in config.toml', type: 'boolean', default: 'true' },
  ],
  'browser': [
    { key: 'enabled', description: 'Enable browser_open tool', type: 'boolean', default: 'true' },
    { key: 'allowed_domains', description: 'Allowed domains for browser_open (empty = all)', type: 'array', example: '["example.com"]' },
    { key: 'backend', description: 'Browser automation backend', type: 'string', default: '"auto"', options: ['auto', 'agent_browser', 'rust_native', 'computer_use'] },
    { key: 'native_headless', description: 'Headless mode for rust-native backend', type: 'boolean', default: 'true' },
    { key: 'native_webdriver_url', description: 'WebDriver endpoint URL for rust-native backend', type: 'string', default: '"http://127.0.0.1:9515"' },
    { key: 'native_chrome_path', description: 'Optional Chrome/Chromium executable path', type: 'string', example: '"/usr/bin/chromium"' },
    { key: 'session_name', description: 'Browser session name for agent-browser automation', type: 'string', example: '"my-session"' },
  ],
  'gui_verification': [
    { key: 'approval_gate', description: 'Approval policy for GUI actions', type: 'string', default: '"supervised_only"', options: ['always', 'supervised_only', 'never'] },
    { key: 'approval_threshold', description: 'Reversibility class that triggers approval', type: 'string', default: '"irreversible"', options: ['partially_reversible', 'irreversible', 'unknown'] },
    { key: 'approval_timeout_secs', description: 'Timeout for native GUI approval prompts', type: 'number', default: '120' },
    { key: 'click_at_preflight', description: 'Strictness of perception preflight gate before click_at', type: 'string', default: '"widget_and_ocr"', options: ['widget_and_ocr', 'widget_only', 'none'] },
  ],
  'web_fetch': [
    { key: 'enabled', description: 'Enable web_fetch tool for fetching page content', type: 'boolean', default: 'false' },
    { key: 'allowed_domains', description: 'Allowed domains (["*"] = all public hosts)', type: 'array', example: '["*"]' },
    { key: 'blocked_domains', description: 'Blocked domains (always takes priority)', type: 'array', example: '["ads.example.com"]' },
    { key: 'max_response_size', description: 'Max response size in bytes', type: 'number', default: '500000' },
    { key: 'timeout_secs', description: 'Request timeout in seconds', type: 'number', default: '30' },
  ],
  'web_search': [
    { key: 'enabled', description: 'Enable web_search tool', type: 'boolean', default: 'false' },
    { key: 'provider', description: 'Search provider', type: 'string', default: '"duckduckgo"', options: ['duckduckgo', 'brave'] },
    { key: 'brave_api_key', description: 'Brave Search API key (required if provider = "brave")', type: 'string', example: '"BSAx..."' },
    { key: 'max_results', description: 'Max results per search (1–10)', type: 'number', default: '5' },
    { key: 'timeout_secs', description: 'Request timeout in seconds', type: 'number', default: '15' },
  ],
  'http_request': [
    { key: 'enabled', description: 'Enable http_request tool for API interactions', type: 'boolean', default: 'false' },
    { key: 'allowed_domains', description: 'Allowed domains for HTTP requests (empty = all denied)', type: 'array', example: '["api.example.com"]' },
    { key: 'max_response_size', description: 'Max response size in bytes', type: 'number', default: '1000000' },
    { key: 'timeout_secs', description: 'Request timeout in seconds', type: 'number', default: '30' },
  ],
  'proxy': [
    { key: 'enabled', description: 'Enable proxy support', type: 'boolean', default: 'false' },
    { key: 'http_proxy', description: 'Proxy URL for HTTP requests (http, https, socks5, socks5h)', type: 'string', example: '"socks5://127.0.0.1:1080"' },
    { key: 'https_proxy', description: 'Proxy URL for HTTPS requests', type: 'string', example: '"socks5://127.0.0.1:1080"' },
    { key: 'all_proxy', description: 'Fallback proxy URL for all schemes', type: 'string', example: '"socks5://127.0.0.1:1080"' },
    { key: 'no_proxy', description: 'No-proxy bypass list', type: 'array', example: '["localhost", "127.0.0.1"]' },
    { key: 'scope', description: 'Proxy application scope', type: 'string', default: '"gloamy"', options: ['environment', 'gloamy', 'services'] },
    { key: 'services', description: 'Service selectors when scope = "services"', type: 'array', example: '["provider.openai"]' },
  ],
  'multimodal': [
    { key: 'max_images', description: 'Max image attachments per request', type: 'number', default: '4' },
    { key: 'max_image_size_mb', description: 'Max image payload size in MiB', type: 'number', default: '5' },
    { key: 'allow_remote_fetch', description: 'Allow fetching remote image URLs', type: 'boolean', default: 'false' },
  ],
  'tunnel': [
    { key: 'provider', description: 'Tunnel provider for public gateway exposure', type: 'string', options: ['cloudflare', 'ngrok', 'custom'] },
    { key: 'token', description: 'Auth token for the tunnel provider', type: 'string', example: '"your-tunnel-token"' },
    { key: 'custom_url', description: 'Custom tunnel URL (when provider = "custom")', type: 'string', example: '"https://myagent.example.com"' },
  ],
  'cost': [
    { key: 'enabled', description: 'Enable cost tracking and budget enforcement', type: 'boolean', default: 'false' },
    { key: 'daily_limit_usd', description: 'Daily spending limit in USD', type: 'number', default: '10.0' },
    { key: 'monthly_limit_usd', description: 'Monthly spending limit in USD', type: 'number', default: '100.0' },
    { key: 'warn_at_percent', description: 'Warn when spending reaches this % of limit', type: 'number', default: '80' },
    { key: 'allow_override', description: 'Allow requests to exceed budget with --override flag', type: 'boolean', default: 'false' },
  ],
  'transcription': [
    { key: 'enabled', description: 'Enable voice transcription for channels that support it', type: 'boolean', default: 'false' },
    { key: 'api_url', description: 'Whisper API endpoint URL', type: 'string', default: '"https://api.groq.com/openai/v1/audio/transcriptions"' },
    { key: 'model', description: 'Whisper model name', type: 'string', default: '"whisper-large-v3-turbo"' },
    { key: 'language', description: 'Language hint (ISO-639-1, e.g. "en", "ru")', type: 'string', example: '"en"' },
    { key: 'max_duration_secs', description: 'Max voice duration in seconds', type: 'number', default: '120' },
  ],
  'tts': [
    { key: 'enabled', description: 'Enable voice synthesis for auto voice replies', type: 'boolean', default: 'false' },
    { key: 'api_key', description: 'TTS-specific API key (preferred over global provider key)', type: 'string', example: '"sk-..."' },
    { key: 'api_url', description: 'OpenAI-compatible speech synthesis endpoint', type: 'string', default: '"https://api.openai.com/v1/audio/speech"' },
    { key: 'model', description: 'TTS model name', type: 'string', default: '"tts-1"' },
    { key: 'voice', description: 'Voice preset name', type: 'string', default: '"alloy"', options: ['alloy', 'echo', 'fable', 'onyx', 'nova', 'shimmer'] },
    { key: 'response_format', description: 'Audio response format', type: 'string', default: '"opus"', options: ['mp3', 'opus', 'aac', 'flac'] },
    { key: 'max_input_chars', description: 'Max input length sent to TTS endpoint', type: 'number', default: '4096' },
    { key: 'voice_reply_mode', description: 'Auto voice-reply policy', type: 'string', default: '"off"', options: ['off', 'voice_only', 'voice_plus_text', 'always'] },
  ],
  'identity': [
    { key: 'format', description: 'Identity format', type: 'string', default: '"openclaw"', options: ['openclaw', 'aieos'] },
    { key: 'aieos_path', description: 'Path to AIEOS JSON file (relative to workspace)', type: 'string', example: '"identity.json"' },
    { key: 'aieos_inline', description: 'Inline AIEOS JSON content', type: 'string', example: '"{...}"' },
  ],
  'reliability': [
    { key: 'max_retries', description: 'Max retry attempts on provider errors', type: 'number', default: '3' },
    { key: 'retry_delay_ms', description: 'Initial retry delay in milliseconds', type: 'number', default: '1000' },
    { key: 'fallback_provider', description: 'Fallback provider ID when primary fails', type: 'string', example: '"openrouter"' },
    { key: 'fallback_model', description: 'Fallback model when primary fails', type: 'string', example: '"gpt-4o-mini"' },
  ],
  'heartbeat': [
    { key: 'enabled', description: 'Enable periodic health pings', type: 'boolean', default: 'false' },
    { key: 'interval_secs', description: 'Heartbeat interval in seconds', type: 'number', default: '60' },
    { key: 'url', description: 'Webhook URL to ping', type: 'string', example: '"https://uptime.example.com/ping/..."' },
  ],
  'skills': [
    { key: 'open_skills_enabled', description: 'Enable community open-skills repository loading', type: 'boolean', default: 'false' },
    { key: 'open_skills_dir', description: 'Path to local open-skills repository', type: 'string', example: '"~/open-skills"' },
    { key: 'prompt_injection_mode', description: 'How skills are injected into system prompt', type: 'string', default: '"full"', options: ['full', 'compact'] },
  ],
  'storage': [
    { key: 'provider', description: 'Storage provider backend (e.g. postgres, sqlite)', type: 'string', options: ['sqlite', 'postgres'] },
  ],
  'peripherals': [
    { key: 'enabled', description: 'Enable peripheral board support (boards become agent tools)', type: 'boolean', default: 'false' },
    { key: 'datasheet_dir', description: 'Path to datasheet docs for RAG retrieval', type: 'string', example: '"docs/datasheets"' },
  ],
  'hardware': [
    { key: 'enabled', description: 'Enable hardware access', type: 'boolean', default: 'false' },
    { key: 'transport', description: 'Transport mode', type: 'string', default: '"none"', options: ['none', 'native', 'serial', 'probe'] },
    { key: 'serial_port', description: 'Serial port path', type: 'string', example: '"/dev/ttyACM0"' },
    { key: 'baud_rate', description: 'Serial baud rate', type: 'number', default: '115200' },
    { key: 'probe_target', description: 'Probe target chip (e.g. "STM32F401RE")', type: 'string', example: '"STM32F401RE"' },
    { key: 'workspace_datasheets', description: 'Enable workspace datasheet RAG for AI pin lookups', type: 'boolean', default: 'false' },
  ],
  'hooks': [
    { key: 'enabled', description: 'Enable lifecycle hook execution', type: 'boolean', default: 'true' },
  ],
  'scheduler': [
    { key: 'enabled', description: 'Enable task scheduler', type: 'boolean', default: 'false' },
  ],
}

const auth = useAuthStore()
const loading = ref(true)
const saving = ref(false)
const error = ref<string | null>(null)
const saveSuccess = ref(false)
const rawConfig = ref('')
const originalConfig = ref('')
const searchQuery = ref('')
const activeSection = ref('All')
const editingValueCell = ref<string | null>(null)
const editDraft = ref('')
const externalChangeDetected = ref(false)
let syncInterval: ReturnType<typeof setInterval> | null = null

const sectionMeta: Record<string, { icon: string; description: string }> = {
  '_root': { icon: 'hugeicons:settings-01', description: 'Core settings' },
  'observability': { icon: 'hugeicons:chart-bar-line', description: 'Logging and tracing' },
  'autonomy': { icon: 'hugeicons:robot-01', description: 'Agent autonomy levels' },
  'security': { icon: 'hugeicons:shield-01', description: 'Security policies' },
  'runtime': { icon: 'hugeicons:computer-terminal-01', description: 'Execution environment' },
  'reliability': { icon: 'hugeicons:refresh', description: 'Retries and fallbacks' },
  'scheduler': { icon: 'hugeicons:clock-01', description: 'Task scheduling' },
  'agent': { icon: 'hugeicons:brain', description: 'Agent orchestration' },
  'skills': { icon: 'hugeicons:sparkles', description: 'Skills loading' },
  'heartbeat': { icon: 'hugeicons:pulse-rectangle-01', description: 'Health pings' },
  'cron': { icon: 'hugeicons:calendar-01', description: 'Cron jobs' },
  'channels_config': { icon: 'hugeicons:chat-01', description: 'Channel settings' },
  'memory': { icon: 'hugeicons:database', description: 'Memory backends' },
  'storage': { icon: 'hugeicons:hard-drive', description: 'Persistent storage' },
  'tunnel': { icon: 'hugeicons:globe', description: 'Public exposure' },
  'gateway': { icon: 'hugeicons:plug-01', description: 'Gateway server' },
  'composio': { icon: 'hugeicons:puzzle', description: 'Composio integration' },
  'one': { icon: 'hugeicons:computer-terminal-02', description: 'One CLI integration' },
  'secrets': { icon: 'hugeicons:key-01', description: 'Secrets encryption' },
  'browser': { icon: 'hugeicons:browser', description: 'Browser automation' },
  'http_request': { icon: 'hugeicons:globe-02', description: 'HTTP requests' },
  'multimodal': { icon: 'hugeicons:image-01', description: 'Image handling' },
  'web_fetch': { icon: 'hugeicons:download-01', description: 'Web fetching' },
  'web_search': { icon: 'hugeicons:search-01', description: 'Web search' },
  'proxy': { icon: 'hugeicons:split', description: 'Proxy settings' },
  'identity': { icon: 'hugeicons:identification', description: 'Identity format' },
  'cost': { icon: 'hugeicons:wallet-01', description: 'Cost tracking' },
  'peripherals': { icon: 'hugeicons:cpu', description: 'Hardware boards' },
  'hardware': { icon: 'hugeicons:chip', description: 'Hardware config' },
  'transcription': { icon: 'hugeicons:mic-01', description: 'Voice transcription' },
  'agents': { icon: 'hugeicons:user-group', description: 'Sub-agent delegates' },
  'hooks': { icon: 'hugeicons:anchor', description: 'Lifecycle hooks' },
  'gui_verification': { icon: 'hugeicons:eye', description: 'GUI verification & approvals' },
  'tts': { icon: 'hugeicons:volume-high', description: 'Text-to-speech' },
}


const hasChanges = computed(() => rawConfig.value !== originalConfig.value)

function cellId(entry: ConfigEntry): string {
  return `${entry.section}:${entry.lineIndex}`
}

function isApiLikeKey(key: string): boolean {
  return key.toLowerCase().includes('api')
}

function isMaskedPlaceholder(value: string): boolean {
  return value.includes('***') || value.includes('•••')
}

function displayValue(entry: ConfigEntry): string {
  if (isApiLikeKey(entry.key)) {
    return '••••••••'
  }
  return entry.value
}

function startValueEdit(entry: ConfigEntry) {
  editingValueCell.value = cellId(entry)
  editDraft.value = isMaskedPlaceholder(entry.value) ? '' : entry.value
}

function clearEditing() {
  editingValueCell.value = null
  editDraft.value = ''
}

function detectType(value: string): ConfigEntry['type'] {
  if (value === 'true' || value === 'false') return 'boolean'
  if (value.startsWith('[')) return 'array'
  if (value.startsWith('{')) return 'object'
  if (/^-?\d+(\.\d+)?$/.test(value)) return 'number'
  return 'string'
}

function parseValue(raw: string): string {
  const trimmed = raw.trim()
  if (trimmed.startsWith('"') && trimmed.endsWith('"')) {
    return trimmed.slice(1, -1)
  }
  if (trimmed.startsWith("'") && trimmed.endsWith("'")) {
    return trimmed.slice(1, -1)
  }
  return trimmed
}

// ── Add Entry Modal ───────────────────────────────────────────────

const showAddModal = ref(false)
const newEntryKey = ref('')
const newEntryValue = ref('')
const newEntrySection = ref('General')
const newEntrySectionValue = ref('')
const sectionDropdownOpen = ref(false)

const sectionSuggestions = computed(() => {
  const q = newEntrySection.value.toLowerCase()
  const allSections = sectionNames.value.filter(n => n !== 'All')
  if (!q) return allSections
  return allSections.filter(n => n.toLowerCase().includes(q))
})

function selectSection(name: string) {
  newEntrySection.value = name
  sectionDropdownOpen.value = false
}

function onSectionBlur() {
  // Delay close so click on option registers first
  setTimeout(() => { sectionDropdownOpen.value = false }, 150)
}

// Schema suggestions for the currently selected section
const sectionSchemaKey = computed(() => {
  const s = newEntrySection.value
  if (s === 'General') return '_root'
  return s
})

const schemaFieldsForSection = computed<SchemaField[]>(() => {
  return SCHEMA_FIELDS[sectionSchemaKey.value] ?? []
})

const selectedSchemaField = computed<SchemaField | null>(() => {
  if (!newEntryKey.value.trim()) return null
  return schemaFieldsForSection.value.find(f => f.key === newEntryKey.value.trim()) ?? null
})

function openAddModal() {
  newEntryKey.value = ''
  newEntryValue.value = ''
  newEntrySectionValue.value = ''
  newEntrySection.value = activeSection.value === 'All' ? 'General' : activeSection.value
  showAddModal.value = true
}

function selectSchemaField(field: SchemaField) {
  newEntryKey.value = field.key
  if (field.default) {
    const raw = field.default.replace(/^"|"$/g, '')
    newEntryValue.value = raw
  } else if (field.example) {
    const raw = field.example.replace(/^"|"$/g, '')
    newEntryValue.value = raw
  }
}

function addNewEntry() {
  if (!newEntryKey.value.trim()) return

  const sectionToUse = newEntrySection.value === 'New Section' ? newEntrySectionValue.value : newEntrySection.value
  if (!sectionToUse) return

  const lines = rawConfig.value.split('\n')
  const targetHeader = `[${sectionToUse}]`
  
  let insertionIndex = -1
  if (sectionToUse === 'General') {
    insertionIndex = lines.findIndex(l => l.trim() !== '' && !l.trim().startsWith('#') && !l.trim().startsWith('['))
    if (insertionIndex === -1) insertionIndex = lines.length
  } else {
    const headerIndex = lines.findIndex(l => l.trim() === targetHeader)
    if (headerIndex !== -1) {
      insertionIndex = lines.findIndex((l, i) => i > headerIndex && l.trim().startsWith('['))
      if (insertionIndex === -1) insertionIndex = lines.length
    } else {
      lines.push('', targetHeader)
      insertionIndex = lines.length
    }
  }

  const formattedValue = detectType(newEntryValue.value) === 'string' && !newEntryValue.value.startsWith('"')
    ? `"${newEntryValue.value}"`
    : newEntryValue.value

  lines.splice(insertionIndex, 0, `${newEntryKey.value} = ${formattedValue}`)
  rawConfig.value = lines.join('\n')
  showAddModal.value = false
}

const sections = computed<ConfigSection[]>(() => {
  const lines = rawConfig.value.split('\n')
  const sectionsMap: Record<string, ConfigEntry[]> = { '_root': [] }
  let currentSection = '_root'

  lines.forEach((line, index) => {
    const trimmed = line.trim()
    
    // Section header
    const sectionMatch = trimmed.match(/^\[([^\]]+)\]$/)
    if (sectionMatch && sectionMatch[1]) {
      currentSection = sectionMatch[1]
      if (!sectionsMap[currentSection]) {
        sectionsMap[currentSection] = []
      }
      return
    }

    // Skip comments and empty lines
    if (trimmed.startsWith('#') || trimmed === '') return

    // Key-value pair
    const kvMatch = trimmed.match(/^([^=]+)=(.*)$/)
    if (kvMatch && kvMatch[1] && kvMatch[2] !== undefined) {
      const key = kvMatch[1].trim()
      const rawValue = kvMatch[2].trim()
      const value = parseValue(rawValue)
      
      if (!sectionsMap[currentSection]) {
        sectionsMap[currentSection] = []
      }
      
      const entries = sectionsMap[currentSection]
      if (entries) {
        entries.push({
          key,
          value,
          type: detectType(rawValue),
          section: currentSection,
          rawLine: line,
          lineIndex: index,
        })
      }
    }
  })

  const result: ConfigSection[] = []
  
  for (const [name, entries] of Object.entries(sectionsMap)) {
    if (entries.length === 0) continue
    const meta = sectionMeta[name] || { icon: 'hugeicons:folder-01', description: 'Configuration section' }
    result.push({
      name: name === '_root' ? 'General' : name,
      icon: meta.icon,
      description: meta.description,
      entries,
    })
  }

  return result
})

const sectionNames = computed(() => {
  return ['All', ...sections.value.map(s => s.name)]
})

const filteredSections = computed(() => {
  let result = sections.value

  if (activeSection.value !== 'All') {
    result = result.filter(s => s.name === activeSection.value)
  }

  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    result = result.map(section => ({
      ...section,
      entries: section.entries.filter(e =>
        e.key.toLowerCase().includes(q) ||
        e.value.toLowerCase().includes(q)
      ),
    })).filter(s => s.entries.length > 0)
  }

  return result
})

function updateEntry(entry: ConfigEntry, newValue: string) {
  const lines = rawConfig.value.split('\n')
  
  // Format value based on type
  let formattedValue = newValue
  if (entry.type === 'string' && !newValue.startsWith('"') && !newValue.startsWith('[') && !newValue.startsWith('{')) {
    formattedValue = `"${newValue}"`
  }
  
  // Replace the line
  const newLine = `${entry.key} = ${formattedValue}`
  lines[entry.lineIndex] = newLine
  rawConfig.value = lines.join('\n')

  clearEditing()
}

function toggleBoolean(entry: ConfigEntry) {
  const newValue = entry.value === 'true' ? 'false' : 'true'
  const lines = rawConfig.value.split('\n')
  lines[entry.lineIndex] = `${entry.key} = ${newValue}`
  rawConfig.value = lines.join('\n')
}

function getTypeIcon(type: ConfigEntry['type']): string {
  switch (type) {
    case 'string': return 'hugeicons:text'
    case 'number': return 'hugeicons:hashtag'
    case 'boolean': return 'hugeicons:toggle-off'
    case 'array': return 'hugeicons:list-view'
    case 'object': return 'hugeicons:code'
    default: return 'hugeicons:question'
  }
}

function getTypeColor(type: ConfigEntry['type']): string {
  switch (type) {
    case 'string': return 'text-sky-500'
    case 'number': return 'text-violet-500'
    case 'boolean': return 'text-emerald-500'
    case 'array': return 'text-amber-500'
    case 'object': return 'text-rose-500'
    default: return 'text-muted-foreground'
  }
}

async function fetchConfig(silent = false) {
  if (!silent) loading.value = true
  error.value = null
  try {
    const response = await auth.fetchWithAuth<{ format: string; content: string }>('/api/config')
    if (silent) {
      // Live sync: only update if we have no unsaved changes and file changed externally
      if (!hasChanges.value && response.content !== originalConfig.value) {
        rawConfig.value = response.content
        originalConfig.value = response.content
        externalChangeDetected.value = true
        setTimeout(() => { externalChangeDetected.value = false }, 3000)
      } else if (hasChanges.value && response.content !== originalConfig.value) {
        // File changed externally while user has unsaved edits — flag it but don't overwrite
        externalChangeDetected.value = true
      }
    } else {
      rawConfig.value = response.content
      originalConfig.value = response.content
    }
  } catch (err: any) {
    if (!silent) error.value = err.message || 'Failed to load configuration'
  } finally {
    if (!silent) loading.value = false
  }
}

function reloadFromFile() {
  externalChangeDetected.value = false
  fetchConfig(false)
}

async function saveConfig() {
  saving.value = true
  error.value = null
  saveSuccess.value = false
  try {
    await auth.fetchWithAuth('/api/config', {
      method: 'PUT',
      headers: { 'Content-Type': 'text/plain' },
      body: rawConfig.value,
    })
    originalConfig.value = rawConfig.value
    saveSuccess.value = true
    externalChangeDetected.value = false
    setTimeout(() => {
      saveSuccess.value = false
    }, 3000)
  } catch (err: any) {
    error.value = err.message || 'Failed to save configuration'
  } finally {
    saving.value = false
  }
}

function resetChanges() {
  rawConfig.value = originalConfig.value
  externalChangeDetected.value = false
}

function handleKeyDown(event: KeyboardEvent) {
  if ((event.metaKey || event.ctrlKey) && event.key === 's') {
    event.preventDefault()
    if (hasChanges.value && !saving.value) {
      saveConfig()
    }
  }
}

onMounted(() => {
  fetchConfig()
  window.addEventListener('keydown', handleKeyDown)
  // Poll for external config file changes every 10 seconds
  syncInterval = setInterval(() => fetchConfig(true), 10_000)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown)
  if (syncInterval !== null) {
    clearInterval(syncInterval)
    syncInterval = null
  }
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden bg-background text-[15px]">
    <!-- Header -->
    <div class="flex-shrink-0 px-6 pt-6 pb-4 border-b border-border/30">
      <div class="flex items-center justify-between mb-4">
        <div class="relative max-w-md">
          <Icon icon="hugeicons:search-01" class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search configuration..."
            class="w-full pl-10 pr-4 py-2 text-[13px] bg-card/50 border border-border/50 rounded-xl focus:outline-none focus:ring-1 focus:ring-primary/50 focus:border-primary/50 text-foreground placeholder:text-muted-foreground"
          />
        </div>

        <div class="flex items-center gap-2">
          <button
            v-if="hasChanges"
            @click="resetChanges"
            class="px-3 py-1.5 text-[12px] font-medium rounded-xl text-muted-foreground hover:text-foreground hover:bg-card/50 transition-colors"
          >
            Discard
          </button>
          <button
            @click="saveConfig"
            :disabled="!hasChanges || saving"
            class="px-4 py-1.5 text-[12px] font-medium rounded-xl transition-all flex items-center gap-1.5"
            :class="hasChanges
              ? 'bg-foreground text-background hover:opacity-90'
              : 'bg-card/50 text-muted-foreground cursor-not-allowed'"
          >
            <Icon v-if="saving" icon="hugeicons:loading-03" class="size-3.5 animate-spin" />
            <Icon v-else-if="saveSuccess" icon="hugeicons:tick-02" class="size-3.5" />
            <Icon v-else icon="hugeicons:floppy-disk" class="size-3.5" />
            {{ saving ? 'Saving...' : saveSuccess ? 'Saved' : 'Save' }}
          </button>
          
          <button
            @click="openAddModal"
            class="px-4 py-1.5 text-[12px] font-medium rounded-xl bg-card/50 text-foreground border border-border/50 hover:bg-card transition-all flex items-center gap-1.5"
          >
            <Icon icon="hugeicons:add-01" class="size-3.5" />
            New Entry
          </button>
        </div>
      </div>

      <!-- Section tabs -->
      <div class="flex items-center gap-1 overflow-x-auto pb-1">
        <button
          v-for="name in sectionNames"
          :key="name"
          @click="activeSection = name"
          class="px-3 py-1.5 text-[12px] font-medium rounded-xl whitespace-nowrap transition-colors"
          :class="activeSection === name
            ? 'text-foreground'
            : 'text-muted-foreground hover:text-foreground hover:bg-card/50'"
        >
          {{ name }}
        </button>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-hidden">
      <div v-if="loading" class="flex-1 flex items-center justify-center h-full">
        <Icon icon="hugeicons:loading-03" class="size-6 animate-spin text-muted-foreground" />
      </div>

      <div v-else-if="error && !rawConfig" class="flex-1 flex items-center justify-center h-full px-6">
        <div class="max-w-md w-full p-6 rounded-lg border border-border/50 bg-destructive/5 text-center">
          <Icon icon="hugeicons:alert-01" class="size-8 text-destructive mx-auto mb-3" />
          <h3 class="text-lg font-medium text-foreground mb-2">Failed to load configuration</h3>
          <p class="text-sm text-muted-foreground mb-4">{{ error }}</p>
          <button
            @click="fetchConfig"
            class="px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:opacity-90 transition-opacity"
          >
            Try Again
          </button>
        </div>
      </div>

      <div v-else class="h-full overflow-auto">
        <!-- Error banner -->
        <div v-if="error" class="sticky top-0 z-10 px-6 py-2 border-b border-destructive/20">
          <div class="flex items-center gap-2 text-[13px] text-destructive">
            <Icon icon="hugeicons:alert-01" class="size-4" />
            <span>{{ error }}</span>
            <button @click="error = null" class="ml-auto hover:opacity-70">
              <Icon icon="hugeicons:cancel-01" class="size-4" />
            </button>
          </div>
        </div>

        <!-- External change banner -->
        <div v-if="externalChangeDetected && !hasChanges" class="sticky top-0 z-10 px-6 py-2 border-b border-border/30">
          <div class="flex items-center gap-2 text-[13px] text-foreground">
            <Icon icon="hugeicons:refresh" class="size-4 text-muted-foreground" />
            <span class="text-muted-foreground">Config file updated externally — reloaded.</span>
            <button @click="externalChangeDetected = false" class="ml-auto hover:opacity-70 text-muted-foreground">
              <Icon icon="hugeicons:cancel-01" class="size-4" />
            </button>
          </div>
        </div>
        <div v-if="externalChangeDetected && hasChanges" class="sticky top-0 z-10 px-6 py-2 border-b border-border/30">
          <div class="flex items-center gap-2 text-[13px]">
            <Icon icon="hugeicons:alert-circle" class="size-4 text-amber-500" />
            <span class="text-amber-600">Config file changed on disk while you have unsaved edits.</span>
            <button @click="reloadFromFile" class="ml-2 text-[12px] font-medium text-foreground underline underline-offset-2 hover:opacity-70">Reload from file</button>
            <button @click="externalChangeDetected = false" class="ml-auto hover:opacity-70 text-muted-foreground">
              <Icon icon="hugeicons:cancel-01" class="size-4" />
            </button>
          </div>
        </div>

        <!-- Table -->
        <div class="px-6 py-4">
          <div v-if="filteredSections.length === 0" class="flex items-center justify-center py-12">
            <p class="text-muted-foreground text-[13px]">No configuration entries found.</p>
          </div>

          <div v-else class="space-y-6">
            <section v-for="section in filteredSections" :key="section.name">
              <div class="flex items-center gap-2 mb-3">
                <Icon :icon="section.icon" class="size-4 text-muted-foreground" />
                <h2 class="text-[16px] font-medium text-foreground">{{ section.name }}</h2>
                <span class="text-[12px] text-muted-foreground">{{ section.description }}</span>
              </div>

              <!-- Table structure -->
              <div class="rounded-xl border border-border/40 overflow-hidden bg-card/20">
                <!-- Header row -->
                <div class="grid grid-cols-[1fr_80px_2fr] gap-4 px-4 py-2.5 bg-card/40 border-b border-border/30 text-[11px] font-medium text-muted-foreground uppercase tracking-wider">
                  <div>Key</div>
                  <div>Type</div>
                  <div>Value</div>
                </div>

                <!-- Data rows -->
                <div
                  v-for="entry in section.entries"
                  :key="`${section.name}-${entry.key}`"
                  class="grid grid-cols-[1fr_80px_2fr] gap-4 px-4 py-3 border-b border-border/20 last:border-0 hover:bg-card/30 transition-colors group"
                >
                  <!-- Key -->
                  <div class="flex items-center gap-2 min-w-0">
                    <span class="font-mono text-[13px] text-foreground truncate" :title="entry.key">
                      {{ entry.key }}
                    </span>
                  </div>

                  <!-- Type badge -->
                  <div class="flex items-center">
                    <span
                      class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-medium bg-card/60"
                      :class="getTypeColor(entry.type)"
                    >
                      <Icon :icon="getTypeIcon(entry.type)" class="size-3" />
                      {{ entry.type }}
                    </span>
                  </div>

                  <!-- Value -->
                  <div class="flex items-center min-w-0">
                    <!-- Boolean toggle -->
                    <template v-if="entry.type === 'boolean'">
                      <button
                        @click="toggleBoolean(entry)"
                        class="flex items-center gap-2 px-3 py-1 rounded-lg transition-colors hover:bg-card/40"
                        :class="entry.value === 'true' ? 'text-emerald-600' : 'text-muted-foreground'"
                      >
                        <Icon
                          :icon="entry.value === 'true' ? 'hugeicons:toggle-on' : 'hugeicons:toggle-off'"
                          class="size-5"
                        />
                        <span class="text-[13px] font-medium">{{ entry.value }}</span>
                      </button>
                    </template>

                    <!-- Editable text input -->
                    <template v-else-if="editingValueCell === cellId(entry)">
                      <input
                        v-model="editDraft"
                        @blur="updateEntry(entry, editDraft)"
                        @keydown.enter.prevent="updateEntry(entry, editDraft)"
                        @keydown.escape.prevent="clearEditing"
                        autofocus
                        class="flex-1 px-3 py-1.5 font-mono text-[13px] bg-card/60 border border-primary/50 rounded-lg focus:outline-none focus:ring-1 focus:ring-primary/50 text-foreground"
                      />
                    </template>

                    <!-- Display value (click to edit) -->
                    <template v-else>
                      <button
                        @click="startValueEdit(entry)"
                        class="flex-1 text-left px-3 py-1.5 font-mono text-[13px] text-foreground bg-transparent hover:bg-card/40 rounded-lg transition-colors truncate group-hover:bg-card/40"
                      >
                        <span v-if="isApiLikeKey(entry.key) || (entry.type === 'string' && isMaskedPlaceholder(entry.value))" class="text-muted-foreground italic">
                          {{ displayValue(entry) }}
                        </span>
                        <span v-else-if="entry.type === 'array' || entry.type === 'object'" class="text-muted-foreground">
                          {{ displayValue(entry).length > 50 ? displayValue(entry).slice(0, 50) + '...' : displayValue(entry) }}
                        </span>
                        <span v-else>{{ displayValue(entry) || '(empty)' }}</span>
                      </button>
                      <button
                        @click="startValueEdit(entry)"
                        class="ml-2 p-1 rounded text-muted-foreground hover:text-foreground hover:bg-card/50 opacity-0 group-hover:opacity-100 transition-all flex-shrink-0"
                        aria-label="Edit value"
                        title="Edit value"
                      >
                        <Icon
                          icon="hugeicons:pencil-edit-01"
                          class="size-3.5"
                        />
                      </button>
                    </template>
                  </div>
                </div>
              </div>
            </section>
          </div>
        </div>

        <!-- Footer -->
        <div class="sticky bottom-0 px-6 py-2 border-t border-border/30 bg-background/95 backdrop-blur flex items-center justify-between text-[11px] text-muted-foreground">
          <div class="flex items-center gap-4">
            <span class="flex items-center gap-1">
              <Icon icon="hugeicons:keyboard" class="size-3.5" />
              <kbd class="px-1.5 py-0.5 bg-card/60 rounded text-[10px]">⌘S</kbd> to save
            </span>
            <span v-if="hasChanges" class="flex items-center gap-1 text-amber-600">
              <Icon icon="hugeicons:pencil-edit-01" class="size-3" />
              Unsaved changes
            </span>
          </div>
          <div class="flex items-center gap-3">
            <span>{{ sections.reduce((acc, s) => acc + s.entries.length, 0) }} entries</span>
            <span>{{ sections.length }} sections</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Add Entry Modal -->
    <Teleport to="body">
      <div
        v-if="showAddModal"
        class="modal-host fixed inset-0 z-50 flex items-end sm:items-center justify-center bg-black/50"
        @click.self="showAddModal = false"
      >
        <div class="modal-panel w-full max-w-2xl mx-4 mb-4 sm:mb-0 rounded-xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
          <!-- Header -->
          <div class="modal-header flex items-center justify-between px-5 py-4 border-b flex-shrink-0">
            <div class="flex items-center gap-2">
              <Icon icon="hugeicons:add-01" class="size-4 modal-text-muted" />
              <span class="text-[14px] font-semibold modal-text">New Config Entry</span>
            </div>
            <button
              @click="showAddModal = false"
              class="size-7 flex items-center justify-center rounded-xl modal-btn-ghost transition-colors"
            >
              <Icon icon="hugeicons:cancel-01" class="size-4 modal-text-muted" />
            </button>
          </div>

          <!-- Body: two-column layout -->
          <div class="flex flex-1 overflow-hidden min-h-0">
            <!-- Left: form -->
            <div class="flex-1 px-5 py-4 space-y-4 overflow-y-auto modal-border-r">
              <!-- Section -->
              <div class="space-y-1.5 relative">
                <label class="text-[11px] font-medium modal-text-muted uppercase tracking-wider">Section</label>
                <input
                  v-model="newEntrySection"
                  type="text"
                  placeholder="e.g. agent, channels_config.telegram"
                  class="modal-input w-full px-3 py-2 text-[13px] font-mono rounded-xl focus:outline-none"
                  @focus="sectionDropdownOpen = true"
                  @blur="onSectionBlur"
                  @input="sectionDropdownOpen = true"
                  autocomplete="off"
                />
                <!-- Custom dropdown -->
                <div
                  v-if="sectionDropdownOpen && sectionSuggestions.length"
                  class="modal-dropdown absolute left-0 right-0 top-full mt-1 z-10 rounded-xl overflow-hidden shadow-lg max-h-52 overflow-y-auto"
                >
                  <button
                    v-for="name in sectionSuggestions"
                    :key="name"
                    type="button"
                    @mousedown.prevent="selectSection(name)"
                    class="modal-dropdown-item w-full text-left px-3 py-2 text-[13px] font-mono"
                  >
                    {{ name }}
                  </button>
                </div>
              </div>

              <!-- Key -->
              <div class="space-y-1.5">
                <label class="text-[11px] font-medium modal-text-muted uppercase tracking-wider">Key</label>
                <input
                  v-model="newEntryKey"
                  type="text"
                  placeholder="e.g. max_tool_iterations"
                  autofocus
                  class="modal-input w-full px-3 py-2 text-[13px] font-mono rounded-xl focus:outline-none"
                />
                <!-- Selected field hint -->
                <div v-if="selectedSchemaField" class="flex flex-col gap-1 pt-1">
                  <p class="text-[12px] modal-text-muted leading-relaxed">{{ selectedSchemaField.description }}</p>
                  <div class="flex flex-wrap gap-1.5 pt-0.5">
                    <span v-if="selectedSchemaField.default" class="text-[11px] modal-text-muted">
                      Default: <code class="modal-code">{{ selectedSchemaField.default }}</code>
                    </span>
                    <span v-if="selectedSchemaField.options" class="text-[11px] modal-text-muted">
                      Options: <code class="modal-code">{{ selectedSchemaField.options.join(' | ') }}</code>
                    </span>
                  </div>
                </div>
              </div>

              <!-- Value -->
              <div class="space-y-1.5">
                <label class="text-[11px] font-medium modal-text-muted uppercase tracking-wider">Value</label>
                <!-- Enum select when options known -->
                <select
                  v-if="selectedSchemaField?.options"
                  v-model="newEntryValue"
                  class="modal-input w-full px-3 py-2 text-[13px] font-mono rounded-xl focus:outline-none"
                >
                  <option value="" disabled>Select an option...</option>
                  <option v-for="opt in selectedSchemaField.options" :key="opt" :value="opt">{{ opt }}</option>
                </select>
                <input
                  v-else
                  v-model="newEntryValue"
                  type="text"
                  :placeholder="selectedSchemaField?.example ? `e.g. ${selectedSchemaField.example}` : 'true, 42, or &quot;some text&quot;'"
                  @keydown.enter="addNewEntry"
                  class="modal-input w-full px-3 py-2 text-[13px] font-mono rounded-xl focus:outline-none"
                />
              </div>
            </div>

            <!-- Right: schema browser -->
            <div class="w-64 flex-shrink-0 flex flex-col overflow-hidden modal-sidebar">
              <div class="px-4 py-3 border-b modal-border-b flex-shrink-0">
                <p class="text-[11px] font-medium modal-text-muted uppercase tracking-wider">
                  Available keys
                  <span v-if="schemaFieldsForSection.length" class="ml-1 normal-case font-normal">for [{{ newEntrySection === 'General' ? 'root' : newEntrySection }}]</span>
                </p>
              </div>
              <div class="flex-1 overflow-y-auto py-1">
                <div v-if="schemaFieldsForSection.length === 0" class="px-4 py-6 text-center">
                  <p class="text-[12px] modal-text-muted">No schema info available for this section.</p>
                  <p class="text-[11px] modal-text-muted mt-1 opacity-60">You can still add custom keys.</p>
                </div>
                <button
                  v-for="field in schemaFieldsForSection"
                  :key="field.key"
                  @click="selectSchemaField(field)"
                  class="w-full text-left px-4 py-2.5 transition-colors modal-field-btn"
                  :class="newEntryKey === field.key ? 'modal-field-active' : ''"
                >
                  <div class="flex items-center gap-2 min-w-0">
                    <span class="font-mono text-[12px] modal-text truncate">{{ field.key }}</span>
                    <span class="text-[10px] flex-shrink-0" :class="getTypeColor(field.type)">{{ field.type }}</span>
                  </div>
                  <p class="text-[11px] modal-text-muted leading-relaxed mt-0.5 truncate">{{ field.description }}</p>
                </button>
              </div>
            </div>
          </div>

          <!-- Footer -->
          <div class="modal-header flex items-center justify-end gap-2 px-5 py-4 border-t flex-shrink-0">
            <button
              @click="showAddModal = false"
              class="px-4 py-1.5 text-[13px] modal-text-muted hover:modal-text modal-btn-ghost rounded-xl transition-colors"
            >
              Cancel
            </button>
            <button
              @click="addNewEntry"
              :disabled="!newEntryKey.trim()"
              class="modal-btn-primary px-4 py-1.5 text-[13px] font-semibold rounded-xl disabled:opacity-30 disabled:cursor-not-allowed transition-all"
            >
              Add Entry
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
kbd {
  font-family: inherit;
}

/* Modal uses explicit CSS variables so it renders correctly in both
   light and dark mode regardless of Teleport placement outside .dark scope */
.modal-host {
  color-scheme: light dark;
}

.modal-panel {
  background-color: var(--card);
  border: 1px solid color-mix(in srgb, var(--border) 60%, transparent);
}

.modal-header {
  border-color: color-mix(in srgb, var(--border) 40%, transparent);
}

.modal-border-r {
  border-right: 1px solid color-mix(in srgb, var(--border) 30%, transparent);
}

.modal-border-b {
  border-bottom-color: color-mix(in srgb, var(--border) 30%, transparent);
}

.modal-sidebar {
  background-color: color-mix(in srgb, var(--card) 60%, var(--background) 40%);
}

.modal-text {
  color: var(--foreground);
}

.modal-text-muted {
  color: var(--muted-foreground);
}

.modal-input {
  background-color: var(--background);
  border: 1px solid color-mix(in srgb, var(--border) 60%, transparent);
  color: var(--foreground);
  transition: box-shadow 0.15s, border-color 0.15s;
}

.modal-input::placeholder {
  color: color-mix(in srgb, var(--muted-foreground) 40%, transparent);
}

.modal-input:focus {
  border-color: color-mix(in srgb, var(--primary) 50%, transparent);
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--primary) 30%, transparent);
}

.modal-code {
  font-family: ui-monospace, monospace;
  font-size: 0.85em;
  color: var(--foreground);
  opacity: 0.8;
}

.modal-btn-ghost {
  color: var(--muted-foreground);
}

.modal-btn-ghost:hover {
  background-color: color-mix(in srgb, var(--muted) 50%, transparent);
  color: var(--foreground);
}

.modal-field-btn {
  border-bottom: 1px solid color-mix(in srgb, var(--border) 20%, transparent);
}

.modal-field-btn:hover {
  background-color: color-mix(in srgb, var(--card) 80%, var(--muted) 20%);
}

.modal-field-active {
  background-color: color-mix(in srgb, var(--primary) 8%, transparent);
}

.modal-btn-primary {
  background-color: var(--foreground);
  color: var(--background);
}

.modal-btn-primary:hover {
  opacity: 0.9;
}

.modal-dropdown {
  background-color: var(--card);
  border: 1px solid color-mix(in srgb, var(--border) 60%, transparent);
}

.modal-dropdown-item {
  color: var(--foreground);
  border-bottom: 1px solid color-mix(in srgb, var(--border) 15%, transparent);
}

.modal-dropdown-item:last-child {
  border-bottom: none;
}

.modal-dropdown-item:hover {
  background-color: color-mix(in srgb, var(--muted) 50%, transparent);
  color: var(--foreground);
}
</style>
