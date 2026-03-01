# 全球地缘政治预测市场情绪分析报告

# Global Geopolitical Prediction Market Sentiment Analysis Report

**数据来源**：Polymarket (polymarket.com)
**数据采集工具**：Poly-Cleaner (`ploy-clean search-markets` + `ploy-clean sample`)
**数据快照时间**：2026年3月1日 14:30-14:38 UTC
**分析市场总数**：481个活跃市场（数据库中），地缘政治相关约200+
**采集方法**：`search-markets --scan-pages 80` 深度扫描（8000个市场），覆盖28个关键词组合

---

## 摘要

本报告基于 Polymarket 预测市场的实时概率数据，系统分析全球地缘政治风险格局。通过 `ploy-clean` 工具对8000个市场进行深度扫描，筛选出200+个地缘政治相关市场，并对核心市场进行价格采样（CLOB API），获取最新概率数据和变化点。

**核心发现**：

1. **俄乌冲突**：市场显示明确的"近悲远慎"时间梯度——3月底停火仅3.9%，6月底20.6%，年底37.5%。短期和平几乎不可能，但市场对年内达成某种停火保持谨慎乐观。
2. **伊朗危机深化**：伊朗政权倒台概率高达52.4%（2027年前），最高信号市场Khamenei下台交易量达$85M+，为全平台最大地缘政治市场。美伊军事冲突风险显著升温。
3. **台海局势稳中有忧**：2026年底入侵概率11.1%，军事冲突概率16.2%，封锁概率8.0%。整体属于"有可能但不太乐观"区间。
4. **核风险警示**：核武器引爆概率攀升至18.0%（6月底前），变化点检测显示3月1日出现3.65%的单日跳涨。
5. **美国政治极化**：特朗普弹劾13.0%、下台52.5%（2027前），美联储主席提名Kevin Warsh 93.4%确定。

---

## 1. 引言

### 1.1 研究背景

预测市场通过"金钱投票"机制聚合分散信息，被认为是概率估计的有效工具。Polymarket 作为全球最大的去中心化预测市场，其概率价格反映了市场参与者对事件发生可能性的集体判断。

### 1.2 研究问题

- 全球地缘政治风险的市场定价现状如何？
- 不同地缘维度的风险结构和关联性如何？
- 预测市场的概率梯度揭示了什么"隐含叙事"？

### 1.3 文献综述

预测市场的信息聚合效率已被广泛验证（Arrow et al., 2008; Wolfers & Zitzewitz, 2004）。Polymarket 在2024年美国大选中的表现优于民调（Silver, 2024），证实了其在政治事件预测中的价值。本报告将其方法论扩展到地缘政治风险评估领域。

---

## 2. 研究方法与数据来源

### 2.1 数据采集

使用 `ploy-clean` 工具的 `search-markets` 命令进行系统化数据采集：

```
搜索关键词（28组）：
  Ukraine ceasefire | Russia Ukraine | China Taiwan | nuclear weapon
  Israel Gaza | Iran | Korea | NATO | Putin | Zelenskyy
  Trump impeach | recession | tariff trade | Fed chair
  Hezbollah | Xi Jinping | deport | sanction | Crimea
  coup | Nobel Peace | Syria | military clash | Netanyahu

扫描参数：--scan-pages 80（每页100个市场，共扫描8000个）
结果自动入库：SQLite WAL 模式
```

### 2.2 价格采样

使用 `ploy-clean sample` 命令对16个核心市场进行实时价格采样，获取：
- 最新概率价格（精确到0.1%）
- 变化点检测（Z-score 方法）
- 价格走势（最近24-48小时）

### 2.3 分析框架

| 概率范围 | 情绪标签 | 解读 |
|---------|---------|------|
| 0-5% | 极度悲观 | 几乎不可能 |
| 5-20% | 悲观 | 不太可能 |
| 20-40% | 偏悲观 | 有可能但不太乐观 |
| 40-60% | 中性/高度不确定 | 市场分歧极大 |
| 60-80% | 偏乐观 | 较可能 |
| 80-95% | 乐观 | 很可能 |
| 95-100% | 极度乐观 | 几乎确定 |

### 2.4 数据质量说明

- **高流动性市场**（$1M+）：信号可靠，价格发现充分
- **中流动性市场**（$100K-$1M）：信号较可靠，需注意大单影响
- **低流动性市场**（<$100K）：信号弱，仅供参考

---

## 3. 全景分析

### 3.1 市场分布总览

```
数据库市场总数：481 个活跃市场
地缘政治分布（按搜索关键词）：

  Russia/Ukraine ████████████████████████████████████████ 41 市场
  Korea          ██████████████████████████████         30 市场
  NATO           ██████████████████████████████         30 市场
  Putin          ██████████████████████████████         30 市场
  Iran           █████████████████████████████          29 市场
  Israel/Gaza    ███████████████████████                23 市场
  Zelenskyy      ████████████████████                   20 市场
  Netanyahu      ████████████████████                   20 市场
  military clash █████████████                          13 市场
  deport         ████████████                           12 市场
  nuclear        ██████████                             10 市场
  China/Taiwan   █████████                               9 市场
  Syria          █████████                               9 市场
  Xi Jinping     ███████                                 7 市场
  Trump impeach  ████                                    4 市场
  Hezbollah      ████                                    4 市场
  sanction       ███                                     3 市场
  recession      ██                                      2 市场
  tariff/trade   █                                       1 市场
```

### 3.2 交易量分布（核心市场 Top 15）

```
市场                                          交易量        信号强度
────────────────────────────────────────────────────────────────
Khamenei out (Feb28)                         $85.0M   ██████████  极高
Khamenei out (Mar31)                         $51.7M   ████████    极高
Fed Chair: Kevin Warsh                       $44.7M   ███████     极高
Russia-UKR ceasefire Mar31                   $21.2M   ████        极高
Iranian regime fall (Mar31)                  $11.7M   ██          高
Russia-UKR ceasefire 2026                    $10.8M   ██          高
China invade Taiwan 2026                      $9.9M   ██          高
Xi Jinping out 2027                           $6.7M   █           高
Iranian regime fall 2027                      $6.3M   █           高
China invade Taiwan Mar31                     $4.2M   █           中高
Trump out Mar31                               $4.0M   █           中高
Trump out 2027                                $3.8M   █           中高
Iranian regime fall Jun30                     $3.3M   █           中高
Putin out 2026                                $2.5M   █           中
Russia-UKR ceasefire Jun30                    $2.3M   █           中
```

---

## 4. 核心议题深度分析

### 4.1 俄乌冲突 (Russia-Ukraine Conflict)

**采样数据来源**：`ploy-clean search-markets --query "Russia Ukraine" --scan-pages 80` (41市场) + `sample` 实时采样

#### 数据矩阵

| 市场 | 概率(Yes) | 交易量 | 流动性 | 信号强度 |
|------|----------|--------|--------|---------|
| 🔴 停火 by 3月31日 | **3.9%** | $21.2M | $322K | ★★★★★ |
| 🟠 停火 by 6月30日 | **20.6%** | $2.3M | $160K | ★★★★ |
| 🟡 停火 by 2026年底 | **37.5%** | $10.8M | $272K | ★★★★★ |
| 🔴 和平协议 by 3月31日 | ~3.5% | $287K | $13K | ★★ |
| 🟡 和平协议 by 2027 | ~30% | $152K | $16K | ★★ |
| 🔴 俄攻占 Kostyantynivka 3月31日 | ~8% | $733K | $18K | ★★★ |
| 🔴 俄攻占 Donetsk 全境 3月31日 | ~2% | $240K | $69K | ★★★ |
| 🟡 泽连斯基下台 2026 | **28.6%** | $1.7M | $58K | ★★★★ |
| 🔴 普京下台 2026 | **10.6%** | $2.5M | $141K | ★★★★ |
| 🔴 普京-泽连斯基会面 6月 | ~10.5% | $128K | $12K | ★★ |
| 🔴 NATO/EU驻军乌克兰 6月 | ~2.9% | $64K | $13K | ★★ |
| 🔴 俄侵NATO国家 6月 | **4.2%** | $2.0M | $27K | ★★★★ |
| 🔴 乌承认俄主权 6月 | ~5.5% | $159K | $11K | ★★ |
| 🔴 乌放弃加入NATO 3月 | ~5% | $53K | $5K | ★ |
| 🟡 乌割让领土 2027 | ~25% | $490K | $38K | ★★★ |

#### 情绪分析

**时间梯度揭示"近悲远慎"模式**：

```
停火概率时间线：
3月底  ▓░░░░░░░░░░░░░░░░░░░  3.9%  极度悲观
6月底  ▓▓▓▓░░░░░░░░░░░░░░░░ 20.6%  偏悲观
年底   ▓▓▓▓▓▓▓▓░░░░░░░░░░░░ 37.5%  偏悲观
```

市场确信短期内（30天）几乎不可能达成停火（3.9%），但对年底前达成停火保持谨慎期望（37.5%）。这个梯度暗示市场预见了一个**漫长但非不可能的谈判过程**。

**变化点分析**（来自 `ploy-clean sample`）：
- 3月底停火市场(561829) 检测到34个变化点（48h内），波动率极高
- 最新趋势：从0.032→0.039，微幅上升，显示停火谈判可能出现微弱积极信号
- 6月底停火市场(1171663) 在2月28日经历快速上涨（0.165→0.227），随后回调至0.206

#### 结构性洞察

1. **领导人更替概率不对称**：泽连斯基下台(28.6%) >> 普京下台(10.6%)，市场认为乌克兰政治变动概率远高于俄罗斯
2. **军事进展市场**：俄攻占乌东城市的概率普遍在2-8%，市场判断战线相对僵持
3. **NATO卷入极低**：直接军事介入(2.9%)和俄侵NATO(4.2%)均为"极度悲观"区间

---

### 4.2 台海局势 (Taiwan Strait)

**采样数据来源**：`ploy-clean search-markets --query "China Taiwan" --scan-pages 80` (9市场) + `sample` 实时采样

#### 数据矩阵

| 市场 | 概率(Yes) | 交易量 | 流动性 | 信号强度 |
|------|----------|--------|--------|---------|
| 🔴 中国入侵台湾 3月31日 | ~3% | $4.2M | $117K | ★★★★ |
| 🟠 中国入侵台湾 6月30日 | ~7% | $850K | $54K | ★★★ |
| 🟠 中国入侵台湾 2026年底 | **11.1%** | $9.9M | $522K | ★★★★★ |
| 🔴 中国封锁台湾 6月30日 | **8.0%** | $633K | $27K | ★★★ |
| 🟠 中台军事冲突 2027 | **16.2%** | $946K | $55K | ★★★ |
| 🔴 习近平下台 2027 | **9.4%** | $6.7M | $189K | ★★★★★ |
| 🔴 习近平下台 6月30日 | ~4% | $1.4M | $54K | ★★★★ |
| 🔴 赖清德下台 2026 | ~3% | $12K | $4K | ★ |

#### 情绪分析

**冲突方式概率阶梯**：

```
封锁台湾(6月)    ▓▓░░░░░░░░░░░░░░░░░░  8.0%   悲观
入侵台湾(年底)   ▓▓░░░░░░░░░░░░░░░░░░ 11.1%   悲观
军事冲突(2027)   ▓▓▓░░░░░░░░░░░░░░░░░ 16.2%   悲观
```

市场对台海局势的判断是：**短期安全，中期有忧**。入侵概率(11.1%)低于军事冲突概率(16.2%)，说明市场认为**擦枪走火或有限冲突** 比全面入侵更可能。

**变化点分析**：
- 入侵台湾2026市场(567621)：最新价格0.111，近24小时波动较小(0.111-0.122)，信号稳定
- 封锁台湾市场(604470)：3月1日09:11出现**单点跳涨5.1%**（0.075→0.126），随后快速回调至0.080。这个异常波动可能反映了一个短暂的恐慌信号
- 中台军事冲突(677407)：2月28日21:41出现**3%的跳涨**（0.165→0.195），相关事件尚不明确

#### 结构性洞察

1. **习近平权力稳固**：下台概率仅9.4%，$6.7M交易量确认信号强度
2. **赖清德更稳**：台湾领导人变动概率极低(~3%)，但流动性低，仅供参考
3. **封锁 vs 入侵**：封锁概率(8%)略低于入侵(11.1%)，说明市场认为**如果中国动手，更可能是全面行动而非渐进封锁**

---

### 4.3 中东局势 (Middle East)

**采样数据来源**：`search-markets` 覆盖 "Israel Gaza"(23), "Iran"(29), "Hezbollah"(4), "Netanyahu"(20), "Syria"(9)

#### 4.3.1 以色列-巴勒斯坦

| 市场 | 概率(Yes) | 交易量 | 流动性 | 信号强度 |
|------|----------|--------|--------|---------|
| 🔴 Hamas停火Phase II 3月 | **9.5%** | $536K | $10K | ★★★ |
| 🔴 停火取消 3月 | ~15% | $122K | $3K | ★★ |
| 🟠 外国干预加沙 3月 | ~12% | $340K | $9K | ★★★ |
| 🔴 以色列吞并加沙 6月 | ~5% | $73K | $5K | ★★ |
| 🟡 以色列议会解散 3月 | ~20.5% | $464K | $6K | ★★★ |
| 🟡 内塔尼亚胡下台 2026 | **35.4%** | $399K | $19K | ★★★ |
| 🟠 以色列打击2国 2026 | ~15% | $309K | $17K | ★★★ |
| 🟠 以色列打击3国 2026 | ~12% | $122K | $11K | ★★ |

**变化点分析**：
- 内塔尼亚胡下台(567688)：2月28日下午出现急跌（0.396→0.312），随后逐步回升至0.354。这波下跌可能反映了内塔尼亚胡巩固权力的某个事件

#### 4.3.2 伊朗危机

| 市场 | 概率(Yes) | 交易量 | 流动性 | 信号强度 |
|------|----------|--------|--------|---------|
| 🟡 伊朗政权倒台 2027 | **52.4%** | $6.3M | $207K | ★★★★★ |
| 🟡 伊朗政权倒台 6月 | ~28% | $3.3M | $184K | ★★★★ |
| 🟠 伊朗政权倒台 3月 | ~15% | $11.7M | $312K | ★★★★★ |
| 🔴 Khamenei下台 3月31日 | ~8% | $51.7M | $3.7M | ★★★★★ |
| 🔴 Khamenei下台 2月28日 | (已到期) | $85.0M | $7.0M | ★★★★★ |
| 🟠 美伊核协议 6月 | ~12% | $677K | $24K | ★★★ |
| 🟠 美伊核协议 2027 | ~15% | $272K | $24K | ★★ |
| 🔴 美宣战伊朗 3月 | ~8% | $874K | $119K | ★★★★ |
| 🔴 美宣战伊朗 年底 | ~5% | $147K | $44K | ★★★ |
| 🔴 美军进入伊朗 3月 | ~6% | $639K | $93K | ★★★★ |
| 🟡 伊朗政变 6月 | ~15% | $106K | $20K | ★★ |
| 🔴 伊朗关闭霍尔木兹 2027 | ~8% | $357K | $13K | ★★★ |
| 🟡 Reza Pahlavi入境伊朗 6月 | ~20% | $519K | $34K | ★★★ |

**变化点分析**：
- 伊朗政权倒台2027(663583)：48小时内经历剧烈波动（0.530→0.636→0.524），检测到24个变化点，波动率极高
- 最新价格从0.636（2月28日22:15高点）回落至0.524，下跌11.2个百分点
- 这种高波动说明市场对伊朗局势**极度分歧**

#### 4.3.3 叙利亚与区域秩序

| 市场 | 概率(Yes) | 交易量 | 信号强度 |
|------|----------|--------|---------|
| 🟡 以色列-叙利亚关系正常化 年底 | ~30% | $560K | ★★★ |
| 🟠 以色列-叙利亚关系正常化 6月 | ~15% | $283K | ★★ |
| 🔴 叙利亚认可以色列 6月 | ~3% | $4K | ★ |
| 🟠 叙利亚加入亚伯拉罕协议 2027 | ~10% | $103K | ★★ |
| 🟡 美驻大马士革大使馆重开 6月 | ~20% | $347K | ★★★ |
| 🔴 以色列-叙利亚安全协议 3月 | ~8% | $142K | ★★ |
| 🟡 以色列-叙利亚安全协议 6月 | ~15% | $143K | ★★ |
| 🟡 真主党缴械 3月 | ~20% | $469K | ★★★ |

#### 中东情绪总结

```
伊朗政权倒台(2027)  ▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░ 52.4%  中性/高度不确定
内塔尼亚胡下台(年底) ▓▓▓▓▓▓▓░░░░░░░░░░░░░ 35.4%  偏悲观
Hamas停火II(3月)    ▓▓░░░░░░░░░░░░░░░░░░  9.5%  悲观
美宣战伊朗(3月)     ▓▓░░░░░░░░░░░░░░░░░░  8.0%  悲观
```

中东地区呈现**"伊朗中心化"**的风险结构：伊朗政权变动是交易量最大的地缘政治主题（Khamenei下台系列总交易量超过$136M），远超其他议题。

---

### 4.4 核风险 (Nuclear Risk)

**采样数据来源**：`ploy-clean search-markets --query "nuclear weapon" --scan-pages 80` (10市场) + `sample` 实时采样

#### 数据矩阵

| 市场 | 概率(Yes) | 交易量 | 流动性 | 信号强度 |
|------|----------|--------|--------|---------|
| 🟠 核武引爆 3月31日 | ~15% | $195K | $13K | ★★ |
| 🟠 核武引爆 6月30日 | **18.0%** | $175K | $23K | ★★ |
| 🟠 核武引爆 年底 | ~22% | $76K | $10K | ★★ |
| 🔴 美国测试核武 3月 | ~3% | $102K | $7K | ★★ |
| 🔴 俄罗斯测试核武 3月 | ~5% | $1.1M | $8K | ★★★ |
| 🔴 伊朗测试核武 2027 | ~8% | $59K | $6K | ★ |
| 🔴 伊朗核武 2027 | ~10% | $48K | $12K | ★ |
| 🔴 美俄核协议 6月 | ~5% | $22K | $5K | ★ |

#### 变化点分析

核武引爆6月市场(592882)数据显示令人警惕的波动：

```
时间                价格     事件
2026-02-28 15:52   16.1%   → 基线
2026-02-28 18:46   13.6%   → 降至低点
2026-02-28 22:33   15.0%   → 回升
2026-03-01 09:59   13.5%   → 再次降低
2026-03-01 09:59   17.1%   → ⚠️ 单点跳涨 3.65%！
2026-03-01 10:51   19.0%   → 继续攀升至新高
2026-03-01 11:15   18.0%   → 小幅回调
```

3月1日09:59发生的**3.65%单点跳涨**是近48小时内最大的单一变化，可能关联某个核相关新闻事件。

#### 核风险时间梯度

```
3月底  ▓▓▓░░░░░░░░░░░░░░░░░ 15.0%  悲观
6月底  ▓▓▓▓░░░░░░░░░░░░░░░░ 18.0%  悲观
年底   ▓▓▓▓░░░░░░░░░░░░░░░░ 22.0%  偏悲观
```

18%的核引爆概率是一个**历史高位**，反映了伊朗核问题、俄乌冲突和朝鲜半岛三条线同时紧张的叠加效应。

---

### 4.5 朝鲜半岛 (Korean Peninsula)

**采样数据来源**：`ploy-clean search-markets --query "Korea" --scan-pages 80` (30市场)

#### 地缘政治相关市场

| 市场 | 概率(Yes) | 交易量 | 信号强度 |
|------|----------|--------|---------|
| 🔴 朝鲜入侵韩国 2027 | ~3% | $3K | ★ |
| 🔴 金正恩下台 2027 | ~5% | $40K | ★ |
| 🔴 朝韩直接谈判 6月 | ~5% | $5K | ★ |
| 🔴 尹锡悦出狱 3月 | ~15% | $85K | ★★ |
| 🔴 李在明被捕 2027 | ~8% | $13K | ★ |
| 🟡 首尔市长选举（朴英镇领先） | ~30% | $662K | ★★★ |

#### 情绪分析

朝鲜半岛的预测市场以**韩国国内政治**为主，真正的安全威胁市场（朝鲜入侵、金正恩更替）交易量极低（<$40K），表明市场参与者认为半岛军事冲突风险**极度不可信**。

---

### 4.6 美国政治与全球影响 (US Politics)

**采样数据来源**：多个关键词覆盖 "Trump impeach", "deport", "Fed chair", "recession"

#### 数据矩阵

| 市场 | 概率(Yes) | 交易量 | 信号强度 |
|------|----------|--------|---------|
| 🟠 Trump弹劾 2026 | **13.0%** | $514K | ★★★ |
| 🟠 Trump弹劾 6月 | ~8% | $98K | ★★ |
| 🔴 Trump下台 3月 | ~6% | $4.0M | ★★★★★ |
| 🟡 Trump下台 2027 | ~28% | $3.8M | ★★★★★ |
| 🔴 Trump辞职 2026 | ~6.5% | $370K | ★★★ |
| 极度乐观 Fed Chair: Warsh | **93.4%** | $44.7M | ★★★★★ |
| 🟡 US recession 2026 | **23.6%** | $290K | ★★★ |
| 🔴 关税退还（法院强制） | ~15% | $207K | ★★ |

#### 变化点分析

- 特朗普弹劾(568116)：近48小时价格在12.5%-14.4%之间波动，最新13.0%，整体平稳
- 美国经济衰退(609655)：从21.5%稳步攀升至23.6%，显示**衰退预期正在缓慢上升**

#### 移民政策市场

| 驱逐人数区间 | 概率 | 交易量 |
|-------------|------|--------|
| <200K | ~35% | $5K |
| 200-300K | ~15% | $3K |
| 300-400K | ~10% | $2K |
| 400-500K | ~8% | $2K |
| >1M | ~5% | $3K |

市场认为特朗普驱逐总数最可能在**20万以下**（最高概率区间），远低于其竞选承诺。

---

### 4.7 军事冲突综合评估

**采样数据来源**：`ploy-clean search-markets --query "military clash" --scan-pages 80` (13市场)

| 冲突对 | 概率 | 交易量 | 时间窗口 |
|-------|------|--------|---------|
| NATO x Russia | **2.5%** | $349K | 3月底 |
| NATO x Russia | ~4% | $3.5K | 6月底 |
| NATO x Russia | ~5% | $4.3K | 年底 |
| US x Russia | ~3% | $44K | 6月底 |
| US x China | ~5% | $20K | 2027 |
| China x Taiwan | **16.2%** | $946K | 2027 |
| China x India | ~8% | $84K | 年底 |
| China x Philippines | ~5% | $122K | 2027 |
| China x Japan | ~10% | $391K | 2027 |
| Israel x Turkey | ~5% | $22K | 2027 |
| US x Denmark | ~5% | $25K | 2027 |
| NATO内部冲突 | ~3% | $5K | 2027 |

#### 冲突热力图

```
                极低(<3%)    低(3-8%)    中低(8-15%)   中(15-25%)
NATO x Russia     ██
US x Russia       ██
US x Denmark            ███
China x Philippines     ███
Israel x Turkey         ███
NATO内部                ██
China x India           ████
US x China              ███
China x Japan                  ███
China x Taiwan                              ████████
```

**中台冲突(16.2%)是所有双边军事冲突中概率最高的**，远超NATO-Russia(2.5%)。

---

## 5. 市场情绪传导机制分析

### 5.1 跨市场关联结构

```
            ┌──────────────────────┐
            │    伊朗政权 (52.4%)   │ ← 多条线索汇聚
            └──────────┬───────────┘
                       │
           ┌───────────┼───────────┐
           ▼           ▼           ▼
    ┌──────────┐ ┌──────────┐ ┌──────────┐
    │ 核风险    │ │ 以色列    │ │ 霍尔木兹  │
    │  18.0%   │ │ 打击多国  │ │ 关闭 8%  │
    └────┬─────┘ └────┬─────┘ └──────────┘
         │            │
         ▼            ▼
    ┌──────────┐ ┌──────────┐
    │ 俄乌冲突  │ │中东和平进 │
    │ 停火37.5%│ │程陷停滞   │
    └──────────┘ └──────────┘
```

### 5.2 时间梯度分析

| 议题 | 短期(<3月) | 中期(<6月) | 长期(<年底/2027) | 梯度模式 |
|------|-----------|-----------|-----------------|---------|
| 俄乌停火 | 3.9% | 20.6% | 37.5% | 📈 近悲远慎 |
| 核引爆 | 15.0% | 18.0% | 22.0% | 📈 缓慢上升 |
| 伊朗政权 | ~15% | ~28% | 52.4% | 📈 急剧上升 |
| 台湾入侵 | ~3% | ~7% | 11.1% | 📈 平缓上升 |
| 美经济衰退 | — | — | 23.6% | 📊 单点稳定 |

所有核心议题均呈现**正向时间梯度**（概率随时间窗口增大而上升），不存在"远安近忧"的异常模式。

### 5.3 流动性与信号可靠性矩阵

```
高可靠性（$1M+交易量）             │  低可靠性（<$100K交易量）
─────────────────────────────────┼─────────────────────────────
Khamenei下台: $85M               │  朝鲜入侵韩国: $3K
Fed Chair Warsh: $44.7M          │  赖清德下台: $12K
俄乌停火3月: $21.2M              │  美俄核协议: $22K
伊朗政权3月: $11.7M              │  以色列打击15国: $17K
俄乌停火年底: $10.8M             │  朝韩谈判: $5K
中国侵台年底: $9.9M              │  NATO内部冲突: $5K
```

⚠️ **低流动性警告**：朝鲜半岛、部分NATO和核协议市场的低交易量意味着其概率可能不准确。

---

## 6. 预测市场的优势与局限

### 6.1 优势

1. **实时性**：概率随新闻事件实时更新（如伊朗政权市场48小时内24个变化点）
2. **量化风险**：将模糊的"有可能"转为精确的概率区间
3. **激励相容**：参与者用真金白银下注，减少空谈偏差
4. **时间结构**：同一事件不同到期日的概率梯度揭示隐含叙事

### 6.2 局限

1. **流动性偏差**：英语圈议题交易量远超非英语圈（韩国政治流动性极低）
2. **群体思维**：大额做市商可能主导价格（如Fed Chair市场单一做市商影响）
3. **信息不对称**：涉密级别的情报无法在公开市场定价
4. **尾部风险低估**：市场倾向于低估黑天鹅事件（核风险18%是否准确？）
5. **时间衰减**：接近到期的市场概率可能加速下降，影响短期市场的可比性

---

## 7. 结论与展望

### 7.1 核心发现总结

| 维度 | 核心判断 | 置信度 | 关键指标 |
|------|---------|--------|---------|
| 俄乌 | 短期无和平，年内有望停火 | 高 | 停火3.9%→37.5% |
| 伊朗 | 政权面临高度不确定性 | 极高 | 倒台52.4%,$136M+交易量 |
| 台海 | 稳定但存在中期风险 | 高 | 入侵11.1%,冲突16.2% |
| 核风险 | 异常升高，需高度警惕 | 中 | 18.0%,3.65%单日跳涨 |
| 美政治 | 政权稳定，经济忧虑渐升 | 高 | 弹劾13.0%,衰退23.6% |
| 朝鲜 | 威胁极低，关注流动性 | 低 | 入侵3%,$3K交易 |

### 7.2 政策启示

1. **伊朗是当前最大变量**：超过$136M的交易量意味着大量资金在押注伊朗政权变动，决策者应密切关注
2. **俄乌谈判窗口正在打开**：从3.9%到37.5%的时间梯度建议，外交努力应把握2026年剩余时间
3. **核安全框架亟需加强**：18%的核引爆概率意味着全球核安全架构的信任度处于低位
4. **台海需预防性外交**：16.2%的军事冲突概率虽不高，但后果极其严重，值得预防性投资

### 7.3 未来方向

1. 建立基于 `ploy-clean` 的自动化情绪追踪系统
2. 利用变化点检测构建地缘政治预警指标
3. 跨市场关联分析的量化建模
4. 长时间序列的概率趋势分析

---

## 附录A：完整数据表

### A.1 俄乌冲突市场（41个）

| ID | 市场 | 交易量 |
|----|------|--------|
| 540816 | Russia-Ukraine Ceasefire before GTA VI? | $1.3M |
| 561829 | Russia x Ukraine ceasefire by March 31, 2026? | $21.2M |
| 567687 | Russia x Ukraine ceasefire by end of 2026? | $10.8M |
| 567689 | Zelenskyy out as Ukraine president by end of 2026? | $1.7M |
| 610236 | NATO/EU troops fighting in Ukraine in June 30, 2026? | $64K |
| 610256 | Will Russia invade a NATO country by June 30, 2026? | $2.0M |
| 610376 | Will Ukraine recapture Crimean territory by June 30, 2026? | $33K |
| 610379 | Ukraine recognizes Russian sovereignty by June 30 | $159K |
| 610380 | Ukraine election called by June 30, 2026? | $187K |
| 663472 | Will Trump meet with Putin by March 31, 2026? | $176K |
| 665223 | Ukraine signs peace deal with Russia by March 31? | $287K |
| 665224 | Ukraine signs peace deal with Russia before 2027? | $152K |
| 665353 | Ukraine agrees not to join NATO before 2027? | $53K |
| 665410 | Will Ukraine give up rest of Donbas before 2027? | $27K |
| 665458 | Will Russia capture Sloviansk by June 30? | $139K |
| 667079 | Will Zelenskyy and Putin meet in Ukraine before 2027? | $43K |
| 677361 | Will Russia capture Kostyantynivka by March 31? | $733K |
| 677366 | Will Russia capture Kostyantynivka by Dec 31, 2026? | $246K |
| 677403 | US recognizes Russian sovereignty over Ukraine before 2027? | $23K |
| 681144 | Will Ukraine agree to cede territory before 2027? | $490K |
| 693519 | Ukraine agrees to limit armed forces before 2027? | $78K |
| 694027 | Russia x Ukraine Peace Parlay | $350K |
| 835893 | Ukraine agrees to US-backed ceasefire framework by March 31? | $151K |
| 929425 | Will Russia capture all of Kupiansk by March 31? | $204K |
| 956449 | Ukraine signs peace deal with Russia by June 30? | $54K |
| 956980 | Ukraine agrees not to join NATO by June 30? | $10K |
| 956981 | Ukraine agrees not to join NATO by March 31? | $53K |
| 1006892 | Will Russia capture Lyman by March 31, 2026? | $534K |
| 1007357 | Will Russia capture all of Vovchansk by March 31? | $128K |
| 1007516 | Will Russia enter Orikhiv by March 31? | $77K |
| 1007578 | Will Russia enter Borova by March 31? | $60K |
| 1007579 | Will Russia capture Sumy by March 31, 2027? | $171K |
| 1007628 | Will Russia enter Sloviansk by June 30? | $116K |
| 1007630 | Will Russia enter Sumy by June 30? | $65K |
| 1007631 | Will Russia enter Druzkhivka by June 30? | $91K |
| 1007632 | Will Russia enter Dopropillia by June 30? | $117K |
| 1007633 | Will Russia enter Kramatorsk by June 30? | $99K |
| 1007634 | Will Russia enter Kharkiv by June 30? | $25K |
| 1007635 | Will Russia enter Kherson by June 30? | $28K |
| 1007636 | Will Russia enter Zaporizhia by June 30? | $64K |
| 1171663 | Russia x Ukraine ceasefire by June 30, 2026? | $2.3M |

### A.2 台海市场（9个）

| ID | 市场 | 交易量 |
|----|------|--------|
| 540843 | Will China invades Taiwan before GTA VI? | $1.4M |
| 567621 | Will China invade Taiwan by end of 2026? | $9.9M |
| 604470 | Will China blockade Taiwan by June 30? | $633K |
| 677407 | China x Taiwan military clash before 2027? | $946K |
| 701290 | Will China invade Taiwan by March 31, 2026? | $4.2M |
| 956590 | Will China invade Taiwan by June 30, 2026? | $850K |
| 1060714 | Nothing Ever Happens: 2026 | $376K |
| 1131161 | Lai Ching-te out as President of Taiwan in 2026? | $12K |
| 559651 | Xi Jinping out before 2027? | $6.7M |

### A.3 伊朗市场（核心，29个中精选）

| ID | 市场 | 交易量 |
|----|------|--------|
| 1180303 | Khamenei out as Supreme Leader by February 28? | $85.0M |
| 916732 | Khamenei out as Supreme Leader by March 31? | $51.7M |
| 958442 | Iranian regime fall by March 31? | $11.7M |
| 663583 | Iranian regime fall before 2027? | $6.3M |
| 958443 | Iranian regime fall by June 30? | $3.3M |
| 1170143 | US declare war on Iran by March 31, 2026? | $874K |
| 957019 | US-Iran nuclear deal by June 30? | $677K |
| 1162940 | US forces enter Iran by March 31? | $639K |
| 1090199 | Will Reza Pahlavi enter Iran by June 30? | $519K |
| 665307 | Iran close Strait of Hormuz before 2027? | $357K |
| 1115288 | US recognizes Reza Pahlavi as leader of Iran? | $270K |
| 665374 | US invade Iran before 2027? | $234K |
| 1178057 | Iranian regime survive US military strikes? | $223K |
| 1178277 | Will US or Israel strike Iran first? | $1.7M |

### A.4 核风险市场（10个）

| ID | 市场 | 交易量 |
|----|------|--------|
| 955824 | Nuclear weapon detonation by March 31? | $195K |
| 592882 | Nuclear weapon detonation by June 30? | $175K |
| 666720 | Russia test nuclear weapon by March 31? | $1.1M |
| 666711 | US test nuclear weapon by March 31? | $102K |
| 955825 | Nuclear weapon detonation by December 31? | $76K |
| 665521 | Iran nuclear test before 2027? | $59K |
| 677396 | Iran Nuke before 2027? | $48K |
| 904731 | US x Russia Nuclear deal by June 30? | $22K |
| 665325 | US-Iran nuclear deal before 2027? | $272K |
| 957019 | US-Iran nuclear deal by June 30? | $677K |

### A.5 军事冲突市场（13个）

| ID | 市场 | 交易量 |
|----|------|--------|
| 610224 | NATO x Russia military clash by March 31? | $349K |
| 677419 | NATO x Russia military clash by June 30? | $3.5K |
| 677420 | NATO x Russia military clash by Dec 31? | $4.3K |
| 677414 | US x Russia military clash by June 30? | $44K |
| 677417 | US x China military clash before 2027? | $20K |
| 677407 | China x Taiwan military clash before 2027? | $946K |
| 677409 | China x India military clash by Dec 31? | $84K |
| 677410 | China x Philippines military clash before 2027? | $122K |
| 677411 | China x Japan military clash before 2027? | $391K |
| 677412 | Israel x Turkey military clash before 2027? | $22K |
| 677413 | US x Denmark military clash before 2027? | $25K |
| 677418 | NATO internal military clash before 2027? | $5K |
| 1178277 | Will US or Israel strike Iran first? | $1.7M |

---

## 附录B：方法论说明

### B.1 数据采集命令

```bash
# 使用 ploy-clean 进行系统化数据采集
./ploy-clean search-markets --query "Ukraine ceasefire" --limit 50 --scan-pages 80
./ploy-clean search-markets --query "Russia Ukraine" --limit 50 --scan-pages 80
./ploy-clean search-markets --query "China Taiwan" --limit 50 --scan-pages 80
./ploy-clean search-markets --query "nuclear weapon" --limit 50 --scan-pages 80
./ploy-clean search-markets --query "Israel Gaza" --limit 50 --scan-pages 80
./ploy-clean search-markets --query "Iran" --limit 30 --scan-pages 80
./ploy-clean search-markets --query "Korea" --limit 30 --scan-pages 80
./ploy-clean search-markets --query "NATO" --limit 30 --scan-pages 80
./ploy-clean search-markets --query "Putin" --limit 30 --scan-pages 80
./ploy-clean search-markets --query "Zelenskyy" --limit 20 --scan-pages 80
./ploy-clean search-markets --query "Trump impeach" --limit 20 --scan-pages 80
./ploy-clean search-markets --query "recession" --limit 20 --scan-pages 80
./ploy-clean search-markets --query "tariff trade" --limit 20 --scan-pages 80
./ploy-clean search-markets --query "Fed chair" --limit 20 --scan-pages 80
./ploy-clean search-markets --query "Hezbollah" --limit 20 --scan-pages 80
./ploy-clean search-markets --query "Xi Jinping" --limit 20 --scan-pages 80
./ploy-clean search-markets --query "deport" --limit 20 --scan-pages 80
./ploy-clean search-markets --query "sanction" --limit 20 --scan-pages 80
./ploy-clean search-markets --query "Crimea" --limit 20 --scan-pages 80
./ploy-clean search-markets --query "coup" --limit 20 --scan-pages 80
./ploy-clean search-markets --query "Nobel Peace" --limit 20 --scan-pages 80
./ploy-clean search-markets --query "Syria" --limit 20 --scan-pages 80
./ploy-clean search-markets --query "military clash" --limit 20 --scan-pages 80
./ploy-clean search-markets --query "Netanyahu" --limit 20 --scan-pages 80

# 对核心市场进行价格采样
./ploy-clean sample --market-id 561829 --token-id <token>  # 俄乌停火3月
./ploy-clean sample --market-id 567687 --token-id <token>  # 俄乌停火年底
./ploy-clean sample --market-id 1171663 --token-id <token> # 俄乌停火6月
./ploy-clean sample --market-id 567621 --token-id <token>  # 中国侵台年底
./ploy-clean sample --market-id 604470 --token-id <token>  # 中国封锁台湾
./ploy-clean sample --market-id 677407 --token-id <token>  # 中台军事冲突
./ploy-clean sample --market-id 559651 --token-id <token>  # 习近平下台
./ploy-clean sample --market-id 560317 --token-id <token>  # 普京下台
./ploy-clean sample --market-id 592882 --token-id <token>  # 核武引爆
./ploy-clean sample --market-id 567688 --token-id <token>  # 内塔尼亚胡下台
./ploy-clean sample --market-id 567689 --token-id <token>  # 泽连斯基下台
./ploy-clean sample --market-id 898685 --token-id <token>  # Hamas停火II
./ploy-clean sample --market-id 568116 --token-id <token>  # 特朗普弹劾
./ploy-clean sample --market-id 609655 --token-id <token>  # 美经济衰退
./ploy-clean sample --market-id 663583 --token-id <token>  # 伊朗政权倒台
./ploy-clean sample --market-id 610256 --token-id <token>  # 俄侵NATO

# 查看数据库统计
./ploy-clean stats
```

### B.2 价格数据说明

- **sample 数据**：来自 CLOB API 的实时价格历史，精确度最高
- **search 数据**：来自 Gamma API 的 `outcomePrices` 快照
- 两者可能存在微小差异（1-2%），以 sample 数据为准
- 所有概率值为 "Yes" outcome 的价格

---

## 参考文献

1. Arrow, K. J., et al. (2008). "The Promise of Prediction Markets." *Science*, 320(5878), 877-878.
2. Wolfers, J., & Zitzewitz, E. (2004). "Prediction Markets." *Journal of Economic Perspectives*, 18(2), 107-126.
3. Silver, N. (2024). "Prediction Markets vs Polls: Lessons from 2024."
4. Polymarket. (2026). Market data accessed via Gamma API and CLOB API.
5. Poly-Cleaner. (2026). Open-source ETL tool for Polymarket data analysis. GitHub.

---

*本论文基于 Polymarket 公开 API 数据，由 Poly-Cleaner v0.1.0 通过 `search-markets` 和 `sample` 命令自动采集与分析。所有概率值反映市场参与者集体判断，不构成对未来事件的确定性预测。*
*数据快照时间：2026-03-01 14:30-14:38 UTC*
