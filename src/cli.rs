// CLI 参数解析

use clap::{Parser, Subcommand};
use std::path::PathBuf;

// ── 主命令 long_about ──────────────────────────────────────────────
const MAIN_LONG_ABOUT: &str = "\
Poly-Cleaner: Polymarket 高性能自适应采样 ETL 工具

从 Polymarket/Gamma API 获取预测市场数据，执行自适应采样与变化点检测，
结果存入 SQLite (WAL 模式)。

数据库表:
  markets        — 市场元数据 (id, question, condition_id, clob_token_ids, ...)
  price_history  — 历史价格  (market_id, token_id, timestamp, price, interval)
  change_points  — 变化点    (market_id, token_id, timestamp, price_before/after)
  sampling_plans — 采样计划  (market_id, token_id, start/end_timestamp, status)

API 端点 (公开，无需认证):
  Gamma API  https://gamma-api.polymarket.com/markets      — 市场列表
  CLOB API   https://clob.polymarket.com/prices-history     — 价格历史

API 约束:
  - Gamma API 无服务端搜索: query/text_query/slug_contains 参数均被忽略，只能客户端过滤
  - CLOB API 偶尔无数据: 部分 token ID 返回空历史，需换另一个 token
  - Gamma 价格快照与 CLOB 实时价格有 1-3% 偏差，精确分析以 sample 为准";

// ── 主命令 after_long_help: 典型工作流 + 研究论文流程 + 常见问题 ──
const MAIN_AFTER_LONG_HELP: &str = "\
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
典型工作流
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  1. 确定研究主题 → 设计关键词集合 (20-30 组，覆盖人/地/事/概念)
  2. 批量搜索     → 每个关键词执行 search-markets --scan-pages 80
  3. 验证入库     → stats 查看总量 + SQL 查询检查覆盖度
  4. 精确采样     → 对核心市场 (高交易量/高相关度) 执行 sample
  5. 数据分析     → 从 markets/price_history/change_points 表提取分析数据

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
研究论文编写流程
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

第一阶段: 数据采集
  1. 设计关键词集合 (20-30 组)
     - 按维度分类: 事件名、地名、人名、组织、概念
     - 每个维度用多种表述覆盖 (如同时搜 \"ceasefire\" 和 \"peace deal\")
     - 所有关键词使用英文
  2. 批量搜索入库
     $ ploy-clean search-markets --query \"<关键词>\" --limit 50 --scan-pages 80
  3. 验证数据完整性
     $ ploy-clean stats
     SQL: SELECT id, question, volume FROM markets WHERE question LIKE '%keyword%' ORDER BY volume DESC;
  4. 对核心市场采样 (选交易量 Top 和高相关度的市场)
     $ ploy-clean sample --market-id <ID> --token-id <TOKEN>

第二阶段: 数据分析
  5. 分维度整理: 按研究主题将市场分组，每组记录 ID/问题/概率/交易量/流动性
  6. 提取关键指标:
     - 概率值: outcome price = 隐含概率
     - 交易量: 信号可靠性 ($1M+ 高可靠 / $100K-$1M 中等 / <$100K 低可靠)
     - 时间梯度: 同一议题不同到期日的概率对比 (短期/中期/长期)
  7. 情绪分类:
       0-5%   极度悲观 (几乎不可能)
       5-20%  悲观 (不太可能)
      20-40%  偏悲观 (有可能但不太乐观)
      40-60%  中性/高度不确定
      60-80%  偏乐观 (较可能)
      80-95%  乐观 (很可能)
      95-100% 极度乐观 (几乎确定)
  8. 跨市场关联: 识别关联结构、时间梯度模式、流动性对信号的影响

第三阶段: 论文撰写
  9. 论文结构:
     # 标题 (中英双语)
     ## 摘要 (300字，核心发现 3-5 点)
     ## 1. 引言 (背景、问题、文献)
     ## 2. 方法与数据 (工具、分析方法、数据质量说明)
     ## 3. 全景分析 (市场分布总览、交易量分布 ASCII 柱状图)
     ## 4. 核心议题深度分析 (每维度一节: 数据矩阵表格 + 情绪分析 + 结构洞察)
     ## 5. 跨市场传导机制 (关联图、时间梯度、流动性矩阵)
     ## 6. 预测市场优势与局限
     ## 7. 结论 (核心发现、政策启示、未来方向)
     ## 附录: 完整数据表
     ## 参考文献
  10. 数据呈现要求:
      - 每个议题必须有数据矩阵表格 (市场/概率/交易量/信号强度)
      - 交易量分布用 ASCII 柱状图
      - 跨市场关联用 ASCII 流程图/树形图
      - 概率精确到小数点后一位，交易量用 $xxM / $xxK 格式
      - 低流动性市场标注信号可靠性警告
  11. 质量检查清单:
      [ ] 所有概率值来自实际 API 数据
      [ ] 注明数据快照时间
      [ ] 低流动性市场标注信号可靠性
      [ ] 时间维度概率梯度分析
      [ ] 跨市场关联分析
      [ ] 预测市场方法论局限性讨论

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
常见问题
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  search 返回 0 结果     → 关键词拼写错误或过于精确，放宽关键词
  sample 返回 0 价格点   → token ID 无 CLOB 数据，换 clob_token_ids 中另一个 token
  概率值差异大           → Gamma 快照 vs CLOB 实时，以 sample 数据为准
  同一市场重复           → 多个关键词命中同一市场，upsert 自动去重
  搜索无特定市场         → 增大 --scan-pages 或换关键词";

// ── search-markets 子命令详细帮助 ──────────────────────────────────
const SEARCH_LONG_ABOUT: &str = "\
搜索市场并自动保存到数据库。

核心机制: Gamma API 不支持服务端关键词搜索。实际工作方式是批量拉取市场列表后
在客户端做 AND 关键词匹配。搜索结果自动 upsert 到 markets 表。";

const SEARCH_AFTER_LONG_HELP: &str = "\
参数选择指南:
  --query       空格分隔的关键词，AND 匹配，必须用英文
  --limit       精准关键词用 20，模糊关键词用 50 (默认 50)
  --scan-pages  每页 100 个市场 (默认 20 页)

扫描量参考:
  --scan-pages 20  → 2000 市场  ~5s   快速验证
  --scan-pages 50  → 5000 市场  ~10s  一般研究
  --scan-pages 80  → 8000 市场  ~18s  推荐值 (80 页后通常已无更多市场)
  --scan-pages 100 → 10000 市场 ~25s  极稀有关键词

  也可在 config/sampling.toml 中永久修改:
  [markets]
  max_scan_pages = 80

关键词设计原则:
  - 双词组合精准度更高: \"Ukraine ceasefire\" 比 \"Ukraine\" 噪声少得多
  - 人名/专有名词最稳定: \"Putin\", \"Bitcoin\" 等直接锚定相关市场
  - 单词适合专属领域: \"Iran\", \"recession\" 等唯一性强的词
  - 宽关键词 + 高 limit: 不确定时用宽关键词 + --limit 50 避免遗漏
  - 多组关键词覆盖同一主题: 从不同角度 (事件名/人名/地名/概念) 搜索，数据库自动去重

示例:
  $ ploy-clean search-markets -q \"Trump\"
  $ ploy-clean search-markets -q \"Bitcoin\" --limit 10
  $ ploy-clean search-markets -q \"nuclear\" --scan-pages 80
  $ ploy-clean search-markets -q \"Ukraine ceasefire\" --limit 50 --scan-pages 80";

// ── sample 子命令详细帮助 ──────────────────────────────────────────
const SAMPLE_LONG_ABOUT: &str = "\
对指定市场执行自适应采样。

核心机制: 通过 CLOB API 获取某个 token 的实时价格历史 (48h 内，1 分钟间隔)，
同时自动运行 Z-score 变化点检测。结果分别写入 price_history 和 change_points 表。
每次返回约 50-200 个价格点 + 10-40 个变化点。";

const SAMPLE_AFTER_LONG_HELP: &str = "\
Token ID 获取方式:
  markets 表的 clob_token_ids 字段是 JSON 数组，如 [\"tokenA\", \"tokenB\"]
  第一个 token 通常对应 \"Yes\" outcome，第二个对应 \"No\"
  SQL: SELECT clob_token_ids FROM markets WHERE id = '<市场ID>'

注意事项:
  - search-markets 返回的 outcomePrices 是 Gamma 快照，可能与 CLOB 实时价格有 1-3% 偏差
  - 需要精确数据时必须用 sample
  - 部分 token 无 CLOB 数据 (返回 0 条记录)，此时换 clob_token_ids 数组中另一个 token

示例:
  $ ploy-clean sample -m 517310 -t <TOKEN_ID>";

// ── stats 子命令详细帮助 ──────────────────────────────────────────
const STATS_LONG_ABOUT: &str = "\
显示数据库统计信息。

显示市场总数、价格记录数、变化点数等汇总信息。
每次大规模搜索或采样后运行，用于验证数据完整性。
可选 --market-id 查看单个市场统计。";

/// Poly-Cleaner: Polymarket 高性能自适应采样 ETL 工具
#[derive(Parser, Debug)]
#[command(name = "ploy-clean")]
#[command(author = "yusteven")]
#[command(version = "0.1.0")]
#[command(about = "High-performance adaptive sampling ETL tool for Polymarket data")]
#[command(long_about = MAIN_LONG_ABOUT)]
#[command(after_long_help = MAIN_AFTER_LONG_HELP)]
pub struct Args {
    /// 配置文件路径
    #[arg(short, long, default_value = "config/sampling.toml")]
    pub config: PathBuf,

    /// 数据库文件路径
    #[arg(short, long, default_value = "polymarket.db")]
    pub database: PathBuf,

    /// 日志级别
    #[arg(short, long, default_value = "info")]
    pub log_level: String,

    /// 子命令
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 获取所有市场列表
    FetchMarkets {
        /// 只获取活跃市场
        #[arg(short, long, default_value_t = true)]
        active: bool,
    },

    /// 获取单个市场详情
    GetMarket {
        /// 市场 ID
        #[arg(short, long)]
        id: String,
    },

    /// 对指定市场执行自适应采样
    #[command(long_about = SAMPLE_LONG_ABOUT, after_long_help = SAMPLE_AFTER_LONG_HELP)]
    Sample {
        /// 市场 ID
        #[arg(short, long)]
        market_id: String,

        /// Token ID (从 markets 表 clob_token_ids JSON 数组获取)
        #[arg(short, long)]
        token_id: String,
    },

    /// 运行完整的 ETL 流程
    Run {
        /// 最大并发市场数
        #[arg(short, long, default_value_t = 16)]
        concurrency: usize,

        /// 只处理指定分类
        #[arg(short, long)]
        category: Option<String>,
    },

    /// 初始化数据库
    InitDb,

    /// 搜索市场并保存到数据库 (客户端 AND 关键词匹配)
    #[command(long_about = SEARCH_LONG_ABOUT, after_long_help = SEARCH_AFTER_LONG_HELP)]
    SearchMarkets {
        /// 搜索关键词 (空格分隔，AND 匹配，必须英文)
        #[arg(short, long)]
        query: String,

        /// 最大返回数量 (默认 50; 精准关键词用 20，模糊用 50)
        #[arg(short, long)]
        limit: Option<usize>,

        /// 扫描页数 (每页 100 市场; 默认 20，推荐 80 覆盖绝大多数活跃市场)
        #[arg(long)]
        scan_pages: Option<usize>,
    },

    /// 显示统计信息
    #[command(long_about = STATS_LONG_ABOUT)]
    Stats {
        /// 市场 ID (不指定则显示全局统计)
        #[arg(short, long)]
        market_id: Option<String>,
    },
}
