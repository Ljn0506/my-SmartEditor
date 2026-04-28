const pptxgen = require("pptxgenjs");
const React = require("react");
const ReactDOMServer = require("react-dom/server");
const sharp = require("sharp");
const { FaBuilding, FaGlobe, FaDatabase } = require("react-icons/fa");

const ICON_COLOR = "2B7DE9"; // 蓝色
const TEXT_COLOR = "2B7DE9"; // 蓝色
const ACCENT_COLOR = "2B7DE9"; // 蓝色

function renderIconSvg(IconComponent, color = "#000000", size = 256) {
  return ReactDOMServer.renderToStaticMarkup(
    React.createElement(IconComponent, { color, size: String(size) })
  );
}

async function iconToBase64Png(IconComponent, color, size = 256) {
  const svg = renderIconSvg(IconComponent, color, size);
  const pngBuffer = await sharp(Buffer.from(svg)).png().toBuffer();
  return "image/png;base64," + pngBuffer.toString("base64");
}

async function main() {
  let pres = new pptxgen();
  pres.layout = 'LAYOUT_16x9';
  pres.title = '安全体系';
  
  let slide = pres.addSlide();
  slide.background = { color: "FFFFFF" };

  // 左侧装饰条 - 3个蓝色矩形块
  slide.addShape(pres.shapes.RECTANGLE, {
    x: 0, y: 1.8, w: 0.15, h: 0.8,
    fill: { color: ACCENT_COLOR }, line: { color: ACCENT_COLOR, width: 0 }
  });
  slide.addShape(pres.shapes.RECTANGLE, {
    x: 0, y: 2.8, w: 0.15, h: 0.5,
    fill: { color: ACCENT_COLOR }, line: { color: ACCENT_COLOR, width: 0 }
  });
  slide.addShape(pres.shapes.RECTANGLE, {
    x: 0, y: 3.5, w: 0.15, h: 0.5,
    fill: { color: ACCENT_COLOR }, line: { color: ACCENT_COLOR, width: 0 }
  });

  // 两侧斜线装饰
  slide.addShape(pres.shapes.LINE, {
    x: 0.8, y: 0.3, w: 0, h: 5.0,
    line: { color: ACCENT_COLOR, width: 2 },
    rotate: 8
  });
  slide.addShape(pres.shapes.LINE, {
    x: 9.2, y: 0.3, w: 0, h: 5.0,
    line: { color: ACCENT_COLOR, width: 2 },
    rotate: -8
  });

  // 生成图标
  const buildingIcon = await iconToBase64Png(FaBuilding, "#" + ICON_COLOR, 256);
  const globeIcon = await iconToBase64Png(FaGlobe, "#" + ICON_COLOR, 256);
  const databaseIcon = await iconToBase64Png(FaDatabase, "#" + ICON_COLOR, 256);

  const items = [
    { icon: buildingIcon, text: "基础设施安全可靠运行" },
    { icon: globeIcon, text: "医院业务安全可信开展" },
    { icon: databaseIcon, text: "数据资源安全可控应用" }
  ];

  const startY = 1.5;
  const spacing = 1.4;
  const iconX = 2.5;
  const textX = 3.5;

  items.forEach((item, index) => {
    const y = startY + index * spacing;
    
    // 添加图标
    slide.addImage({
      data: item.icon,
      x: iconX, y: y, w: 0.7, h: 0.7
    });
    
    // 添加文字
    slide.addText(item.text, {
      x: textX, y: y, w: 5, h: 0.7,
      fontSize: 24,
      fontFace: "Microsoft YaHei",
      color: TEXT_COLOR,
      bold: true,
      align: "left",
      valign: "middle"
    });
  });

  await pres.writeFile({ fileName: "安全体系.pptx" });
  console.log("PPT generated: 安全体系.pptx");
}

main().catch(console.error);
