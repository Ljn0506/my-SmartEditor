const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.launch({
    headless: true,
    executablePath: '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
    args: ['--no-proxy-server']
  });
  const page = await browser.newPage();

  const errors = [];
  page.on('console', msg => {
    const type = msg.type();
    if (type === 'error' || type === 'warning') {
      errors.push(`[${type}] ${msg.text()}`);
    }
  });
  page.on('pageerror', err => {
    errors.push(`[pageerror] ${err.message}`);
  });

  // ====== Mock Tauri APIs ======
  await page.addInitScript(() => {
    window.__TAURI__ = {
      invoke: async (cmd, args) => {
        console.log('[MOCK TAURI]', cmd, args);

        // Tauri v2 dialog plugin uses plugin:dialog|open
        if (cmd === 'plugin:dialog|open') {
          return '/tmp/test-requirement.docx';
        }

        if (cmd === 'parse_document') {
          return {
            text: '第一段测试内容\n第二段测试内容\n第三段测试内容',
            paragraphs: [
              { index: 0, text: '第一段测试内容', char_offset: 0 },
              { index: 1, text: '第二段测试内容', char_offset: 7 },
              { index: 2, text: '第三段测试内容', char_offset: 14 }
            ]
          };
        }

        if (cmd === 'extract_requirements') {
          return {
            requirements: [
              { id: 1, text: '系统应支持基于角色的访问控制（RBAC）', certainty: 'certain', selected: true },
              { id: 2, text: '系统应支持等保三级要求 [需确认]', certainty: 'uncertain', selected: false },
            ]
          };
        }

        if (cmd === 'check_deviation_items') {
          return {
            items: [
              { id: 1, section: '技术要求', requirement_text: '支持RBAC', response_text: '完全支持', status: 'None', risk_level: 'low', explanation: '完全响应' }
            ],
            html_report: '<p>共检查 1 项，无偏离</p>',
            json_report: '{}',
            markdown_report: '# 偏离检查报告\n\n共检查 1 项，无偏离'
          };
        }

        if (cmd === 'check_fatal_risks_text') {
          return [];
        }

        if (cmd === 'check_self_review_async') {
          return {
            issues: [
              { id: 1, category: 'format', sub_category: 'punctuation', severity: 'Warning', description: '标点符号使用不规范', paragraph_index: 0, auto_fixable: true, original: '，，', suggestion: '，' }
            ]
          };
        }

        if (cmd === 'apply_self_review_fixes') {
          return {
            mode: 'Copy',
            output_path: '/tmp/test_fixed.docx',
            backup_path: null,
            changes: [{ paragraph_index: 0, original: '，，', modified: '，' }]
          };
        }

        if (cmd === 'export_deviation_report_markdown') {
          return '# 偏离检查报告\n\n测试报告内容';
        }

        if (cmd === 'get_global_params') {
          return null;
        }

        if (cmd === 'get_cards') {
          return [];
        }

        if (cmd === 'get_ai_config') {
          return {
            provider: 'Ollama',
            base_url: 'http://localhost:11434',
            model: 'qwen2.5:14b',
            api_key: null
          };
        }

        if (cmd === 'update_ai_config') {
          return;
        }

        if (cmd === 'write_clipboard_html') {
          return;
        }

        throw new Error('Unknown mock command: ' + cmd);
      },
      core: { invoke: async (cmd, args) => window.__TAURI__.invoke(cmd, args) },
      dialog: { open: async () => '/tmp/test-requirement.docx' }
    };
    window.__TAURI_INTERNALS__ = window.__TAURI__;
  });

  const results = {
    title: null,
    errors: [],
    tests: []
  };

  // ====== Test 1: 首页加载 ======
  await page.goto('http://localhost:1420/');
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(2000);

  results.title = await page.title();
  await page.screenshot({ path: '/Users/ljn/smart-editor/smart-editor-app/e2e-01-initial.png', fullPage: true });
  results.tests.push({ name: '首页加载', status: results.title === '智能文档助手' ? 'pass' : 'fail' });

  // ====== Test 2: 需求上传流程 ======
  try {
    // 点击"需求上传"tab（默认就是）
    const uploadTab = await page.$('button:has-text("需求上传")');
    if (uploadTab) await uploadTab.click();
    await page.waitForTimeout(500);

    // 点击文件上传区域触发选择文件
    const dropZone = await page.$('text=/拖入文件到这里|点击选择/');
    if (dropZone) await dropZone.click();
    await page.waitForTimeout(1000);

    // 验证文件列表显示
    const fileItem = await page.$('text=/test-requirement.docx/');
    results.tests.push({ name: '需求上传-文件选择', status: fileItem ? 'pass' : 'fail' });

    // 点击"开始分析"
    const analyzeBtn = await page.$('button:has-text("开始分析")');
    if (analyzeBtn) {
      await analyzeBtn.click();
      await page.waitForTimeout(2000);
    }

    // 验证需求列表显示
    const reqItem = await page.$('text=/RBAC|访问控制/');
    results.tests.push({ name: '需求上传-需求提取', status: reqItem ? 'pass' : 'fail' });

    await page.screenshot({ path: '/Users/ljn/smart-editor/smart-editor-app/e2e-02-upload.png', fullPage: true });
  } catch (e) {
    results.tests.push({ name: '需求上传流程', status: 'fail', error: e.message });
  }

  // ====== Test 3: 校对流程 ======
  try {
    const checkTab = await page.$('button:has-text("校对")');
    if (checkTab) {
      await checkTab.click();
      await page.waitForTimeout(1000);
    }

    // 点击"选择文件"按钮（投标文档）
    const selectDocBtn = await page.$('button:has-text("选择文件")');
    if (selectDocBtn) {
      await selectDocBtn.click();
      await page.waitForTimeout(1000);
    }

    // 验证文件已加载
    const docLoaded = await page.$('text=/test-requirement.docx/');
    results.tests.push({ name: '校对-文件加载', status: docLoaded ? 'pass' : 'fail' });

    // 点击"检查投标文件"
    const runCheckBtn = await page.$('button:has-text("检查投标文件")');
    if (runCheckBtn) {
      await runCheckBtn.click();
      await page.waitForTimeout(2000);
    }

    // 验证结果显示（mock 返回有偏离风险项和自查问题）
    const checkResult = await page.$('text=/完全响应|偏离风险|自查问题|共检查/');
    results.tests.push({ name: '校对-检查结果', status: checkResult ? 'pass' : 'fail' });

    await page.screenshot({ path: '/Users/ljn/smart-editor/smart-editor-app/e2e-03-check.png', fullPage: true });
  } catch (e) {
    results.tests.push({ name: '校对流程', status: 'fail', error: e.message });
  }

  // ====== Test 4: 智能生成流程 ======
  try {
    const generateTab = await page.$('button:has-text("智能生成")');
    if (generateTab) {
      await generateTab.click();
      await page.waitForTimeout(1000);
    }

    // 验证生成面板加载
    const generatePanel = await page.$('text=/智能生成|生成章节大纲/');
    results.tests.push({ name: '智能生成-面板加载', status: generatePanel ? 'pass' : 'fail' });

    await page.screenshot({ path: '/Users/ljn/smart-editor/smart-editor-app/e2e-04-generate.png', fullPage: true });
  } catch (e) {
    results.tests.push({ name: '智能生成流程', status: 'fail', error: e.message });
  }

  results.errors = errors;
  console.log(JSON.stringify(results, null, 2));

  await browser.close();
})();
