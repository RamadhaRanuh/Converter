"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const path_1 = __importDefault(require("path"));
const opencvService_1 = __importDefault(require("./opencvService"));
const psdService_1 = __importDefault(require("./psdService"));
const svgService_1 = __importDefault(require("./svgService"));
const aiService_1 = __importDefault(require("./aiService"));
class ConverterService {
    async convertImage(filePath, targetFormat, options = {}) {
        const ext = path_1.default.extname(filePath).toLowerCase();
        try {
            // Route to appropriate service based on file extension
            switch (ext) {
                case '.psd':
                    return await psdService_1.default.convertPSD(filePath, targetFormat, options.quality);
                case '.svg':
                    if (targetFormat === 'svg') {
                        return await svgService_1.default.optimizeSVG(filePath);
                    }
                    else {
                        // Convert SVG to other formats using OpenCV
                        return await opencvService_1.default.convertImage(filePath, targetFormat, options.quality);
                    }
                case '.ai':
                    return await aiService_1.default.convertAI(filePath, targetFormat);
                case '.jpg':
                case '.jpeg':
                case '.png':
                case '.bmp':
                case '.webp':
                case '.tiff':
                    return await opencvService_1.default.convertImage(filePath, targetFormat, options.quality);
                default:
                    throw new Error(`Unsupported file format: ${ext}`);
            }
        }
        catch (error) {
            console.error('Conversion error:', error);
            throw error;
        }
    }
    async extractPSDLayers(filePath, targetFormat = 'png') {
        if (path_1.default.extname(filePath).toLowerCase() !== '.psd') {
            throw new Error('Can only extract layers from PSD files');
        }
        return await psdService_1.default.extractLayers(filePath, targetFormat);
    }
}
exports.default = new ConverterService();
