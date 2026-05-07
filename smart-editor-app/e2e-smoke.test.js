const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.launch({
    headless: true,
    executablePath: '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'
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

  // Mock Tauri APIs before navigating
  await page.addInitScript(() => {
    window.__TAURI__ = {
      invoke: async (cmd, args) => {
        console.log('[MOCK TAURI]', cmd, args);
        if (cmd === 'parse_document') {
          return {
            text: '第一段\n第二段\n第三段',
            paragraphs: [
              { index: 0, text: '第一段', char_offset: 0 },
              { index: 1, text: '第二段', char_offset: 3 },
              { index: 2, text: '第三段', char_offset: 6 }
            ]
          };
        }
        if (cmd === 'check_deviation_items') {
          return {
            items: [],
            html_report: '<p>无偏离</p>',
            json_report: '{}',
            markdown_report: '无偏离'
          };
        }
        if (cmd === 'check_fatal_risks_text') {
          return { risks: [], html_report: '', json_report: '{}', markdown_report: '' };
        }
        if (cmd === 'check_self_review_async') {
          return {
            issues: [
              { id: 1, category: 'format', severity: 'minor', description: '测试标点', paragraph_index: 0, auto_fixable: true, original: '，，', suggestion: '，' }
            ]
          };
        }
        if (cmd === 'apply_self_review_fixes') {
          return '/tmp/test_fixed.docx';
        }
        throw new Error('Unknown mock command: ' + cmd);
      },
      core: { invoke: async (cmd, args) => window.__TAURI__.invoke(cmd, args) },
      dialog: { open: async () => ({ path: '/tmp/test.docx' }) }
    };
    window.__TAURI_INTERNALS__ = window.__TAURI__;
  });

  await page.goto('http://127.0.0.1:5173/');
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(2000);

  // Screenshot 1: initial load
  await page.screenshot({ path: '/Users/ljn/smart-editor/smart-editor-app/e2e-01-initial.png', fullPage: true });

  // Check title
  const title = await page.title();

  // Try clicking "CheckTab" if it exists
  const checkTab = await page.$('text=/检查|校审|review/i');
  if (checkTab) {
    await checkTab.click();
    await page.waitForTimeout(1000);
    await page.screenshot({ path: '/Users/ljn/smart-editor/smart-editor-app/e2e-02-checktab.png', fullPage: true });
  }

  console.log(JSON.stringify({
    title,
    errors,
    hasCheckTab: !!checkTab
  }, null, 2));

  await browser.close();
})();
