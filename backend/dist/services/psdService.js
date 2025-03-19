"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const psd_1 = __importDefault(require("psd"));
const path_1 = __importDefault(require("path"));
const sharp_1 = __importDefault(require("sharp"));
const fs_1 = __importDefault(require("fs"));
class PSDService {
    // Convert PSD to PNG
    async convertPSD(inputPath, outputFormat, quality = 90) {
        const psd = await psd_1.default.open(inputPath);
        const outputPath = this.generateOutputPath(inputPath, outputFormat);
        // Convert to PNG first
        const pngBuffer = psd.image?.toPng();
        // Use sharp for final conversion
        await (0, sharp_1.default)(pngBuffer)
            .toFormat(outputFormat, { quality })
            .toFile(outputPath);
        return outputPath;
    }
    // Generate output path
    async extractLayers(inputPath, outputFormat) {
        const psd = await psd_1.default.open(inputPath);
        const outputDir = path_1.default.join(path_1.default.dirname(inputPath), path_1.default.basename(inputPath, path_1.default.extname(inputPath)) + '-layers');
        if (!fs_1.default.existsSync(outputDir)) {
            fs_1.default.mkdirSync(outputDir, { recursive: true });
        }
        const layers = [];
        const tree = psd.tree();
        const processNode = async (node, index) => {
            if (node.isGroup()) {
                for (let i = 0; i < node.children().length; i++) {
                    await processNode(node.children()[i], `${index}-${i}`);
                }
            }
            else {
                if (node.visible) {
                    const layerPath = path_1.default.join(outputDir, `layer-${index}.${outputFormat}`);
                    await (0, sharp_1.default)(node.toPng())
                        .toFormat(outputFormat)
                        .toFile(layerPath);
                    layers.push({
                        name: node.name,
                        path: layerPath
                    });
                }
            }
        };
        for (let i = 0; i < tree.children().length; i++) {
            await processNode(tree.children()[i], i);
        }
        return {
            layersDir: outputDir,
            layers
        };
    }
    generateOutputPath(inputPath, outputFormat) {
        const dirName = path_1.default.dirname(inputPath);
        const baseName = path_1.default.basename(inputPath, path_1.default.extname(inputPath));
        return path_1.default.join(dirName, `${baseName}-converted.${outputFormat}`);
    }
}
exports.default = new PSDService();
