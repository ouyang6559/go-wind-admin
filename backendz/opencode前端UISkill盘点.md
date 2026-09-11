# opencode 前端 UI 优化 Skill 盘点

> 检阅目录：`C:\Users\pauls\.config\opencode\skills\`（全部已注册 Skill 共 1000+）
> 本文件筛选其中**可用于前端 UI 设计、优化、动效、视觉 QA、性能**的 Skill，按类别汇总成表。
> 说明：`anim-*`（Animate.css 单键帧）、`xyz-*`（AnimXYZ）、`fx-*`（Canvas 特效）、`frame-*`（动效视频帧）、三个 `threejs-*` 家族内同类条目已合并为"家族行"以控制篇幅；其余均按单个 Skill 逐一列出。

---

## 目录

1. [综合前端设计/开发体系](#1-综合前端设计开发体系)
2. [设计体系与风格语言（防 AI 味 / taste）](#2-设计体系与风格语言防-ai-味--taste)
3. [预设视觉风格模板（开箱即用设计系统）](#3-预设视觉风格模板开箱即用设计系统)
4. [动画与动效引擎](#4-动画与动效引擎)
5. [WebGL / Canvas 背景与视觉特效](#5-webgl--canvas-背景与视觉特效)
6. [微交互 UI 组件动效（单点增强）](#6-微交互-ui-组件动效单点增强)
7. [动效库家族（批量组件）](#7-动效库家族批量组件)
8. [CSS / 样式技术工具](#8-css--样式技术工具)
9. [前端性能、视觉 QA 与测试](#9-前端性能视觉-qa-与测试)
10. [图片与图标资源](#10-图片与图标资源)
11. [推荐组合用法](#11-推荐组合用法)

---

## 1. 综合前端设计/开发体系

| Skill | 核心特点 | 适用场景 |
|---|---|---|
| **frontend** | 前端/UI/UX/视觉总入口，路由 4 套规则集（设计品味+品牌参照、Playwright/Lighthouse/Core Web Vitals 完美化、ui-ux-db 色板字体指南、designpowers 无障碍/评审/交接）+ 真实网站截图研究（lazyweb） | 一切前端 UI 工作的第一站：建站、改版、美化、性能审计、视觉 QA |
| **frontend-design** | 反模板化的克制设计指导：美学方向、排版、避免"默认值即丑" | 需要设计方向但不想套模板时 |
| **web-design-engineer** | 用 HTML/CSS/JS/React 构建打磨过的可视化产物：页面、Dashboard、原型、幻灯片、动画、UI 稿、数据可视化 | 交付浏览器渲染的前端可视化成果物 |
| **web-artifacts-builder** | 用 React + Tailwind + shadcn/ui 构建复杂多组件 HTML 工件，支持状态管理、路由 | 需要复杂交互/多组件的大型单页工件 |
| **web-design-guidelines**（openmontage） | 按 Web Interface Guidelines 审查 UI 代码：可访问性、UX、最佳实践 | "review my UI / check accessibility / audit design" |
| **web-clone** | 网站复刻决策树：先拿真源码→判路径→逆向拆解→搭工程→换内容，覆盖静态/内容站/重前端三类 | 克隆、仿站、还原某站交互与视觉 |
| **web-to-design-md** | 从真实网站提取设计令牌（色板/字体/间距/CSS/动效）→ 结构化 design.md，供 AI 消费 | 从现有站点"扒设计规范"喂给动效/生成流程 |
| **web-shader-extractor** | 从线上页面抽取真实 WebGL/WebGPU/Canvas/shader 效果并移植为独立轻量 JS 项目 | 复刻网站上的 GPU 特效（流体、metaball、gaussian-splat 等） |
| **design-consultation**（gstack） | 理解产品→研究竞品→提出完整设计系统（美学/字体/颜色/布局/间距/动效）+ 字体颜色预览 | 前期设计咨询、定设计系统 |
| **design-html**（gstack） | 将设计定稿落地为生产级 HTML/CSS | 设计到代码的收尾环节 |
| **design-review**（gstack） | 设计师视角 QA：发现视觉不一致、间距层级问题、AI 味、慢交互并修复 | 改版后的设计师级自查 |
| **design-shotgun**（gstack） | 批量生成多个 AI 设计变体→对比面板→收集结构化反馈→迭代 | 探索多种设计方向、A/B 视觉比选 |
| **design-is** | 以 Dieter Rams 十条准则审计设计，逐条评分+证据，输出 NEW/REFINE/REDESIGN 结论与改版 Prompt | 设计评审并推动改版计划 |
| **design-first-ui-prompting** | 设计优先、spec 驱动的 UI 生成提示词模板（目标/格式/布局/字体/色彩/约束/负向） | 写出生图/AI 生成的一致 UI prompt |
| **hallmark** | 反 AI 味设计：从零建页、审计、改版、从 URL/截图提取设计方向 | 新页面/改版/设计提取 |
| **huashu-design** | 用 HTML 做高保真原型、交互 Demo、幻灯片、动画、设计变体探索 + 设计方向顾问 + 专家评审；可导出 MP4/GIF | 交互原型、Hi-fi Demo、动画演示、变体比选、评审 |
| **theme-factory** | 10 套预设主题（配色+字体）+ 即时生成新主题，可应用到幻灯片/文档/报告/HTML 落地页 | 快速给成果物套主题 |
| **landing-page** | 高转化落地页结构/布局/转化策略/文案/SEO·AEO/常见坑 | 设计或重写单产品落地页 |
| **pricing-page** | 高转化定价页：方案结构、文案、SEO/AEO、FAQ、布局、实验 | 设计或重写 SaaS 定价页 |
| **style-pack-skill-main**（风格包） | 从参考图/剧照/视频提取视觉风格 DNA 打包成风格包，生成色卡 + 文生图/视频提示词表格 | 需要视觉风格提取与风格化提示词时 |

---

## 2. 设计体系与风格语言（防 AI 味 / taste）

| Skill | 核心特点 | 适用场景 |
|---|---|---|
| **design-taste-frontend**（taste-skill） | 反模板化前端：先解读简报推断设计方向，重建时不套模板，pre-flight 校验 | 落地页/作品集/改版 |
| **design-taste-frontend-v1** | 原 v1 版，行为与默认版不同，仅向后兼容用 | 依赖旧行为的项目 |
| **gpt-taste** | 精英级 UX/UI + GSAP 动效工程师：Python 驱动真随机布局、AIDA 结构、大字报排版、无缝隙 bento 栅格、GSAP ScrollTrigger | 高质感、动态丰富的落地页 |
| **high-end-visual-design**（soft-skill） | 像顶级 agency 一样设计：精确字体/间距/阴影/卡片/动画，屏蔽常见 AI 廉价默认值 | 追求"高级感/昂贵感"的页面 |
| **industrial-brutalist-ui** | 工业机能风：Swiss 印刷 × 军用终端美学，刚性栅格、极端字号对比、模拟磨损 | 数据面板、作品集、编辑型网站 |
| **minimalist-ui** | 干净编辑风：暖单色、排版对比、平铺 bento、柔和粉彩，无渐变无重阴影 | 极简编辑型界面 |
| **redesign-existing-projects** | 存量站点升级到高级质量：审计当前设计、识别 AI 泛化模式、应用高标准且不破坏功能 | 现有站/项目改版升级 |
| **stitch-design-taste** | 面向 Google Stitch 的语义设计系统，生成 agent 友好的 DESIGN.md，执行高级反泛化标准 | Stitch 生态或需要 DESIGN.md 规范 |
| **image-to-code** | 先自行生成设计图再深度分析，按图一比一实现网站 | 图生代码、追求与设计稿一致 |
| **imagegen-frontend-web** | 生成优质、面向转化的网站设计参考图（每个 section 一张图，保持全局色板一致） | 用 AI 图定前端视觉方向 |
| **imagegen-frontend-mobile** | 生成 App 原生质感的多屏概念图（iOS/Android/跨端，含手机框） | 移动端界面设计参考 |
| **apple-design-skill-main**（apple-liquid-glass） | 苹果级 Web UI："macOS Liquid Glass" 美学，半透玻璃/发丝线/负字距，含令牌、组件、图标、自检 | 要"苹果风/液态玻璃"效果的页面 |
| **brand-guidelines** | 应用 Anthropic 官方品牌色与字体到任意产物 | 需 Anthropic 品牌观感 |
| **brandkit** | 高端品牌视觉包生成：品牌规范板、Logo 系统、品牌形象册、视觉世界演示 | 品牌视觉系统设计 |

---

## 3. 预设视觉风格模板（开箱即用设计系统）

| Skill | 核心特点 |
|---|---|
| **agency-grid-layout-minimal** | 极简 agency 网格：编辑式栅格、超大字号、克制的全大写工具标签 |
| **blue-cloudy-clean-modern** | 清亮蓝天空氛围：柔和云光、极简白框、静谧高级排版 |
| **blue-laser-clean-glass-layout** | 深色玻璃系统：细蓝激光氛围、磨砂高级外壳、面板结构 |
| **book-serif-index** | 档案书卷风：衬线正文、等宽索引、做旧纸感、页边批注 |
| **bright-green-tech-system-webgl** | 亮绿科技：分栏硬边框深色面、等宽标签、WebGL 可视化区 |
| **clean-minimal-beige-light-mode** | 米色浅色极简：暖中性外壳、静默流程网格、克制强调色 |
| **dark-blue-contrasting-clean** | 深蓝高对比：钴蓝渐变特性块、硬朗边框、克制辉光 |
| **dark-glass-clean-layout** | 深色玻璃：磨砂外壳、多栏工作台、浮动数据卡 |
| **dither-laser-dark-mode** | 深色高级系统：近黑面 + 有序抖动纹理 + 细激光强调 |
| **editorial-tech** | 编辑杂志 × 产品科技：非对称栅格、电影横幅、等宽标签 |
| **framed-tech-dark-border-gradient** | 深色科技：边框渐变外壳、非对称网格面板、等宽标签 |
| **funky-purple-container-tech** | 深色容器科技：品红紫强调、层叠圆角外壳、有趣未来感焦点物 |
| **glass-dark-mode-clock** | 深色玻璃 + 校准刻度盘 + 精密科幻仪表框 |
| **high-contrast-skeuomorphic-clean** | 高对比拟物：塑形深色面、锐利明暗分离、触感内凹深度 |
| **image-first-grid-layout** | 图驱网格：满幅摄影、结构引导线、锚定内容块 |
| **light-mode-paper-technical** | 浅色技术：暖纸面、深色外框、斜纹纹理、括号几何 |
| **mesh-gradient-dark-blue-clean** | 深蓝网格渐变：近黑底色、程序化蓝雾、极简结构、星球纵深 |
| **nested-container-clean-agency** | 嵌套容器 agency：编辑外壳 + 内嵌深色特性块 + 圆角高级卡 |
| **nested-container-frames** | 容器嵌套帧系统：外容器边界 + 内嵌容器圆角分层 |
| **orange-clean-paper-saas** | 纸感 SaaS：暖中性、橙色强调、圆角高级表单 |
| **split-layout-technical** | 技术分屏：双面板、细框线、等宽元数据、克制的编辑排版 |
| **tech-green-dark-mode-modern** | 现代深色科技：磨砂黑面、翠绿信号强调、等宽标签、发光卡 |
| **technical-wireframe-info-layout** | 单色线框：分解 3D 结构、连接注释、稀疏信息标签 |
| **industrial-brutalist-ui** | 工业机能风（见第 2 节） |
| **minimalist-ui** | 极简编辑风（见第 2 节） |

> 特点：均为"一套完整视觉系统 + 排版/色板/组件约定"，直接给 AI 当设计规范使用；可配合 `theme-factory` 换色。

---

## 4. 动画与动效引擎

| Skill | 核心特点 | 适用场景 |
|---|---|---|
| **gsap** | GSAP 综合技能：Timeline、ScrollTrigger、stagger、transform，含性能与坑 | 一切 GSAP 动效 |
| **gsap-core** | 官方核心 API：to/from/fromTo、缓动、duration、matchMedia（响应式/减弱动效） | 基础补间动画 |
| **gsap-react** | React/Next 中 useGSAP、refs、context、cleanup | React 动效 |
| **gsap-frameworks** | Vue/Svelte 等非 React 框架：生命周期、作用域选择器、卸载清理 | 非 React 框架动效 |
| **gsap-timeline** | 时间线：位置参数、嵌套、播放控制、动画编排 | 序列化动画编排 |
| **gsap-plugins** | 官方插件：ScrollTo、ScrollSmoother、Flip、Draggable、SplitText、CustomEase 等 | 高级动效能力 |
| **gsap-scrolltrigger** | 滚动驱动：scrub、pin、触发器 | 滚动动效、视差、吸顶 |
| **gsap-scrolltrigger-storytelling** | 电影感 sticky 叙事：渐进 UI 揭示、滚动同步、沉浸过渡 | 产品故事页 |
| **gsap-performance** | 性能优化：优先 transform、避免 layout thrashing、will-change | 动画卡顿优化 |
| **gsap-utils** | gsap.utils：clamp/mapRange/random/snap/toArray 等工具函数 | 数学辅助动效 |
| **framer-motion** | React 中以迪士尼 12 动画原则实现 Framer Motion 动效 | React 组件动效 |
| **animation-systems** | 产品级动效系统（Stripe/Linear/Apple/Vercel 水准）：运动原则、缓动时长默认值、编排、scroll/hover、性能、无障碍 | 需要"大厂质感"动效 |
| **motion-anything** | 一句话意图→产出精致动效：自动分类意图、按 MOTION-SPEC 选配方（timing/easing/克制预算），覆盖网页/幻灯片/展示页/视频 | 动效总入口、快速产出 |
| **cinematic-gsap-lenis-motion-system** | GSAP + ScrollTrigger + Lenis 平滑滚动的电影级系统：字距揭示、视差、吸顶、磁吸 hover、自定义光标 | 高端编辑型/获奖级官网 |
| **cinematic-scroll-storytelling** | Lenis 平滑 + GSAP ScrollTrigger 滚动叙事：滚动同步、sticky 卡堆栈、视差背景、预加载 | 滚动故事型落地页 |
| **masked-reveal** | 滚动进入视口时词级遮罩渐显（overflow mask + GSAP） | 标题/hero 词级揭示 |
| **staggered-word-reveal** | 词级编辑感渐显（fade+rise，IntersectionObserver） | 高级标题、hero 文案 |
| **animation-on-scroll** | 基于 IntersectionObserver 的滚动触发，Tailwind 友好 | 通用滚动揭示 |
| **scroll-reveal** | 进入视口渐显上升，支持 stagger，尊重减弱动效 | 长页面节奏揭示 |
| **optimize-web-animations** | 剖析/审计/优化前端动画性能：内存泄漏、CSS/canvas/WebGL rAF、marquee/skeleton、timer/listener 泄漏 | 页面越用越卡、掉帧排查 |

---

## 5. WebGL / Canvas 背景与视觉特效

### 5.1 GPU Shader 背景（均为"营销级 hero 背景"，一页一个，文字后垫 scrim，减弱动效时渲染静态帧）

| Skill | 视觉特点 |
|---|---|
| **aurora / soft-aurora / iridescence** | 极光 / 柔和极光 / 虹彩流体质感 |
| **balatro / prism / prismatic-burst** | 棱镜折射 / 分光 / 爆裂棱彩 |
| **dark-veil / faulty-terminal** | 暗色幕布 / 故障终端 |
| **dither / grainient** | Bayer 抖动噪波 / 颗粒渐变 |
| **ferrofluid / silk / threads** | 铁磁流体 / 丝绸 / 丝线编织 |
| **galaxy / lightfall / lightning / light-rays / side-rays** | 星系 / 光落 / 闪电 / 光柱 / 侧向射线 |
| **gradient-blinds / plasma / plasma-wave** | 渐变百叶 / 等离子 / 等离子波 |
| **line-waves / waves / strands** | 线条波场 / Perlin 曲线（含指针尾迹）/ 流苏 |
| **liquid-chrome / orb / particles** | 液态铬 / 光球 / 3D 粒子云 |
| **noise** | 电影感噪点覆盖层 |
| **pixel-blast / pixel-snow / ripple-grid** | 像素爆发 / 像素雪 / 波纹栅格 |
| **splash-cursor** | 指针拖拽的 WebGL 流体染色模拟 |

### 5.2 场景化背景与 3D

| Skill | 核心特点 | 适用场景 |
|---|---|---|
| **background-grid-webgl** | 透视 WebGL 网格 + 粒子雾 + 慢速前进 + 相机视差 | 科技感 hero 背景 |
| **corner-lasers** | 四角锚定激光：细光束、亮发射节点、辉光、雾 | 科幻/数据面板角标装饰 |
| **atmosphere-background** | 深色氛围：纵向飘动光褶 + 屏幕混合辉光 + 角落辉光 | 深色页面氛围底 |
| **dot-field / dot-grid** | 点阵（指针外凸）/ 交互点阵（靠近辉光、快速移动推挤、点击冲击波） | 交互式科技背景 |
| **globe-particles** | 3D 粒子星球：致密发光核心 + 稀薄环带 | 行星/数据地球效果 |
| **webgl-3d-object** | 真实 3D WebGL 物体：几何网格深度、PBR 材质、方向光+环境光、透视相机、浮动旋转 | 3D hero 物体/产品展示 |
| **webgl-laser** | 全屏细激光背景：白热竖核 + 品牌色光晕 + 烟雾 | 激光背景特效 |
| **webgl-landing-steering** | 引导 WebGL 重落地页走向特定视觉效果（高级/科技/趣味/电影感），兼顾转化、性能、复杂度 | WebGL 落地页方向把控 |
| **threejs**（openmontage 家族） | threejs-fundamentals（场景/相机/层级）、threejs-animation（骨骼/变形/混合）、threejs-geometry（几何/缓冲/实例化）、threejs-lighting（灯光/阴影/IBL）、threejs-materials（PBR/自定义着色器）、threejs-loaders（GLTF/纹理/HDR 加载）、threejs-textures（UV/环境贴图）、threejs-interaction（raycast/控制器/输入）、threejs-postprocessing（bloom/DOF/后期）、threejs-shaders（GLSL/ShaderMaterial） | 一切 Three.js 3D 场景 |
| **cobejs** | 轻量交互地球（canvas 设置、标记、交互、React/Next 集成） | 3D 地球点缀 |
| **globe-gl** | globe.gl（WebGL/ThreeJS）3D 地球数据可视化：点/弧/多边形/标签层 | 3D 地球数据可视化 |
| **vantajs** | Vanta.js WebGL 动态背景：参数、resize、性能、React/Next 集成 | 快速加 WebGL 背景 |
| **matterjs** | Matter.js 2D 物理：引擎/刚体/约束/碰撞 | 物理交互（掉落、碰撞） |

---

## 6. 微交互 UI 组件动效（单点增强）

> 均为 CSS/JS 单组件微动效，绝大多数尊重 `prefers-reduced-motion`。

| Skill | 效果 | 使用注意 |
|---|---|---|
| **attention-pulse** | 软扩散光环，吸引到唯一元素（CTA/角标/新点） | 一屏一个 |
| **border-beam** | 亮色彗星沿边框环绕 | 单个"新/精选"元素 |
| **bounce-cards** | 卡片堆栈弹跳展开 | 3–5 张一组 |
| **bottom-sheet** | 底部抽屉滑出 + 背景压暗 | 移动端反馈 |
| **button-press** | 按钮按下弹簧下沉回弹 | 有状态变化的控件 |
| **card-lift-hover** | 可点卡片悬停上浮 + 阴影加深 | 仅真正可点的卡片 |
| **checkbox-pop** | 复选框勾选弹出动画 | 表单反馈 |
| **circular-text** | 环形文字缓慢旋转（徽章/印章） | 纯装饰，短语短 |
| **click-spark** | 点击飞溅火花 | 主行动按钮，勿用于严肃操作 |
| **count-up / counter** | 数字滚动到目标 / 里程表式数字 | 统计/指标块 |
| **dock** | macOS 风格 Dock 图标磁放大 | 一屏一个 |
| **elastic-slider** | 弹性滑块（弹簧填充+旋钮） | 单个突出控件 |
| **electric-border** | 抖动电光描边 | 单一 hero 元素 |
| **glare-hover** | 悬停扫过柔和高光 | 少数 hero 卡 |
| **glass-icons** | 玻璃拟态图标按钮悬停发光 | 工具栏/社交行 |
| **gooey-nav / gooey-blob-system** | 胶囊滑块弹性合并 / SVG 融合粘液系统 | 分段导航 / 有机流体形 |
| **gradient-text** | 品牌渐变在标题内缓慢流动 | 单句短语 |
| **gradual-blur** | 向边缘渐进模糊，暗示"更多内容" | 滚动列表底部 |
| **kinetic-headline** | 标题词/字母依次入场 | 单 hero 标题 |
| **like-burst** | 点赞按钮触发粒子爆发 | 用户点击时触发，勿加载时触发 |
| **loading-shimmer** | 骨架屏柔和扫光 | 真实加载中，勿作装饰 |
| **magnetic-button** | 主按钮磁性吸附光标后回弹 | 1–2 个主行动 |
| **magnet-lines** | 线条栅格集体指向光标 | hero 背景，抓眼球 |
| **marquee-loop** | 无缝无限跑马灯 | 内容横滚 |
| **pixel-card** | 像素瓦片波浪点亮（悬停） | 少量卡 |
| **pixel-transition** | 悬停时像素爆发切换背面 | 少量卡 |
| **rotating-text** | 词语原位轮换 | 一屏一个，词要短 |
| **reveal-hover-effect** | 光标跟随聚光灯，径向遮罩揭示第二图像 | hover 显色/前后对比/材质 |
| **ripple / ripple-press** | Material 风格水波反馈 / 按点水波 | 有状态控件，勿用于破坏性操作 |
| **scroll-float** | 词级上浮入视口 | 1–2 行，位移克制 |
| **shine-border** | 柔和光带沿边框缓行 | 一屏一个 hero 卡 |
| **shimmer-button** | 主 CTA 光带扫过（静置慢、悬停快） | 单一主按钮 |
| **shiny-text** | 强调色文字上柔和高光扫过 | 小标签，1–2 个 |
| **spotlight-card** | 径向聚光跟随光标 + 边框提亮 | 特性/定价卡 |
| **stagger-list** | 列表项依次上升渐显 | 首屏短列表/导航 |
| **star-border** | 柔和光点绕胶囊按钮旋转 | 单 CTA |
| **stepper** | 步骤指示器填充推进 | 多步流程（结账/引导） |
| **tab-bar-slide** | 底部标签胶囊滑动 | 移动 App 反馈 |
| **tilt-3d** | hero 卡/封面 3D 倾斜 + 柔和辉光 | 1 张或几张 hero 卡 |
| **toast-pop** | Toast 底部滑入→停留→淡出 | 有状态变化 |
| **toggle-spring** | 开关旋钮弹簧横跳 | 设置/表单开关 |
| **fade-content / fade-in-up** | 柔和上浮渐显（加载/入视口） | 常规内容揭示 |
| **falling-text** | 字母逐个下落 | 一行短标题 |
| **blur-text** | 单词从模糊到清晰逐个点亮 | 高级标题入场 |
| **text-scramble / shuffle-text / decrypted-text / glitch-text** | 乱码解码 / 字母洗牌 / 解密 / RGB 故障分裂 | 科技感标题/标签，1 行 |
| **true-focus** | 词逐个聚焦点亮（其余模糊） | 短短语引导视线 |
| **crosshair / target-cursor / blob-cursor / image-trail** | 十字准星光标 / 瞄准环光标 / 粘液拖尾光标 / 缩略图拖尾光标 | 自定义光标，一页一个 |
| **dock / stepper / ripple / toast-pop** 等 | 详见上表 | 仅用于真实状态变化 |

---

## 7. 动效库家族（批量组件）

| 家族 | 成员 | 特点与用法 |
|---|---|---|
| **anim-\***（约 80 个） | anim-fadein/\*、anim-bounce\*、anim-slide\*、anim-flip\*、anim-zoom\*、anim-rotate\*、anim-roll\*、anim-back\*、anim-shake\*、anim-pulse、anim-swing、anim-tada、anim-heartbeat、anim-jello、anim-wobble、anim-hinge、anim-flash、anim-jackinthebox、anim-lightspeed\*、anim-headshake 等 | 即插即用 Animate.css 键帧动画（Hippocratic-2.1，© Daniel Eden），纯 CSS 无 JS，尊重减弱动效；克制使用并保留署名 |
| **xyz-\***（9 个） | xyz-fade-big/down/left/right/small/up、xyz-flip-left/up、xyz-rise-big、xyz-rotate | AnimXYZ 可组合入场：`xyz="fade up-3"` 声明式混合 fade/move/scale/rotate，免写 keyframes |
| **fx-\***（20 个） | fx-confetti、fx-firework、fx-particle-burst、fx-shockwave、fx-sparkle-trail、fx-starfield、fx-matrix-rain、fx-neural-net、fx-knowledge-graph、fx-constellation、fx-orbit-ring、fx-galaxy-swirl、fx-gradient-blob、fx-magnetic-field、fx-data-stream、fx-counter-explosion、fx-letter-explode、fx-typewriter-multi、fx-word-cascade、fx-chain-react | Canvas 特效包（Open Design html-ppt），用于幻灯片/发布视频/页面点缀；一屏一个、克制使用 |
| **frame-\***（13 个） | frame-bold-poster、frame-bold-signal、frame-build-minimal、frame-creative-voltage、frame-data-chart-nyt、frame-data-rollup、frame-electric-studio、frame-glitch-title、frame-light-leak-cinema、frame-liquid-bg-hero、frame-logo-outro、frame-pentagram-stat、frame-takram-organic | Remotion/HyperFrames 动效帧模板：海报字、数据滚动、分屏引用、Logo 结尾、胶片漏光等，用于视频动效帧 |
| **lottie-\*** | lottie-fab、lottie-favorite、lottie-pagination、lottie-tab（组件 JSON）+ **lottie-bodymovin**（AE 导出规范）+ **text-to-lottie**（手写 Lottie JSON） | 跨端一致的可移植 JSON 动效（~160KB 运行时），用于 FAB/点赞/分页/标签等 |
| **vfx-text-cursor** | 光标光轨 + 色差射线 + 方向光斑，词级引用揭示 | 视频片头引用文案 |

---

## 8. CSS / 样式技术工具

| Skill | 核心特点 | 适用场景 |
|---|---|---|
| **tailwindcss** | Tailwind 布局/排版/响应式/主题/组件模式快速配方 + 常见坑（content 路径、动态类名、@apply 滥用） | 一切 Tailwind 界面 |
| **tailwind-design-system**（openmontage） | 用 Tailwind v4 建可扩展设计系统：设计令牌、组件库、响应式模式 | 组件库/设计系统 |
| **beautiful-shadows** | 精确 Tailwind 任意值阴影：分层的精致中性层级 | 卡片/面板/弹层高级阴影 |
| **css-border-gradient** | 渐变描边（伪元素 mask）精致边缘高光 | 卡/定价面板/导航/弹窗 |
| **css-alpha-masking** | linear-gradient 水平/垂直边缘淡出（mask-image） | 边缘渐隐 |
| **corner-diagonals** | 对角切角/倒角边缘 | 科技感按钮/卡片/面板 |
| **container-lines** | 竖向容器测量引导线 + 角标方块 | 精准结构化布局 |
| **framed-grid-layout** | 极简框线栅格：可见边界线、L 形角标、斜纹 | 编辑/技术/指南式布局 |
| **number-details** | 01/02/03 数字装饰标记 | 编号视觉元素 |
| **glass-dark-ui** | 深色玻璃拟态：可读对比、磨砂、渐变边框 | 玻璃卡/深色 hero |
| **skeuomorphic-ui** | 拟物：层叠渐变、内外阴影、微纹理、压印文字 | 按压/雕刻/软胶质感 |
| **progressive-blur** | 多层 CSS 渐进模糊（backdrop-filter masks） | 顶部/底部渐进模糊 |
| **ant-design / antd** | antd 6.x / ProComponents / Ant Design X 选型、主题令牌、SSR、a11y、性能、本地 API 查询 | Ant Design 项目 |
| **unicorn-studio** | 嵌入与定制 Unicorn Studio 交互动画：响应式、性能、分层、回退 | 品牌交互动画嵌入 |
| **company-logos** | 用 Iconify Simple Icons（64x64）logo 代替文字 | logo 墙 |
| **solar-duotone-bold** | Iconify Solar Duotone Bold 图标风格 | 图标统一风格 |

---

## 9. 前端性能、视觉 QA 与测试

| Skill | 核心特点 | 适用场景 |
|---|---|---|
| **visual-qa** | 双 Oracle 视觉 QA：脚本证据（像素 diff/相似度/hotspot/TUI 溢出检查）+ 两个只读评审 pass（设计系统真实性 / 视觉与 CJK 缺陷），产出 good/bad 结论 | 构建/改动任何 UI 后必做 |
| **webapp-testing** | Playwright 与本地产物交互测试：验证前端功能、调试 UI、截图、看日志 | 本地 Web 应用验证 |
| **qa / qa-only**（gstack） | 系统化 QA 测试 Web 应用并修复 / 仅报告 | 上线前 QA |
| **frontend（perfection 部分）** | Playwright/Chromium Lighthouse/Core Web Vitals 性能审计 | 性能与 CWV |
| **optimize-web-animations** | 动画性能剖析：内存泄漏、rAF、离屏暂停、CPU/GPU 占用 | 页面卡顿/越用越慢 |
| **web-design-guidelines** | 可访问性/UX 准则审查 | 无障碍与最佳实践 |
| **design-review**（gstack） | 设计师视角 QA + 修复 | 视觉细节自查 |
| **webgl-landing-steering** | 转化/性能/复杂度/视觉平衡引导 | WebGL 页风险把控 |
| **performance-profiling** | Apple 平台（Instruments/Xcode/MetricKit）性能分析 | 仅 iOS/macOS 原生，非 Web |

---

## 10. 图片与图标资源

| Skill | 核心特点 | 适用场景 |
|---|---|---|
| **unsplash-asset-images** | 选高质量 Unsplash 图并输出真实 URL + 分辨率/宽高比指导（1:1/4:5/3:4/16:9/9:16） | 头像、背景、壁纸素材 |
| **aura-asset-images** | Aura Assets 库存风图（按标签搜索，返回 5 个真实 URL + 分辨率建议） | 设计稿/营销图素材 |
| **company-logos** | Iconify 品牌 logo | 信任 logo 墙 |
| **solar-duotone-bold** | 统一风格图标 | 界面图标 |

---

## 11. 推荐组合用法

```text
新建页面/改版 UI
  → frontend（总入口路由）
  → hallmark / design-taste-frontend / web-to-design-md（设计方向 + 参考）
  → 从第 3 节选一套视觉模板 或 theme-factory 换肤
  → landing-page / pricing-page（若为转化页）
  → gsap / motion-anything / animation-systems（动效）+ 第 6/7 节单点微交互
  → WebGL 背景：第 5 节选一个 hero 背景
  → 完成后 visual-qa + web-design-guidelines + frontend(perf) 验收

复刻/移植某个网站特效
  → web-clone（整站） / web-shader-extractor（单 GPU 特效） / web-to-design-md（设计令牌）

存量页面变卡/掉帧
  → optimize-web-animations + gsap-performance + frontend(perf)

React 项目动效
  → framer-motion / gsap-react + 第 6 节微交互
```

### 使用优先级建议

1. **必用**：`frontend`（总路由）、`visual-qa`（交付前验证）、`web-design-guidelines`（可访问性）
2. **高价值**：`hallmark`、`design-taste-frontend`、`gsap`、`motion-anything`、`optimize-web-animations`
3. **按需**：预设风格模板、WebGL 背景、微交互组件、`design-is`（评审推动改版）

---

*生成时间：2026-08-14 · 依据 `C:\Users\pauls\.config\opencode\skills\` 全部 Skill 的 SKILL.md 描述筛选整理*
