"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const path_1 = __importDefault(require("path"));
const opencv4nodejs_1 = __importDefault(require("opencv4nodejs"));
class OpenCVService {
    async convertImage(inputPath, outputFormat, quality = 90) {
        const image = await opencv4nodejs_1.default.imreadAsync(inputPath);
        const outputPath = this.generateOutputPath(inputPath, outputFormat);
        const compressionParams = [
            outputFormat === 'jpg' ? opencv4nodejs_1.default.IMWRITE_JPEG_QUALITY : opencv4nodejs_1.default.IMWRITE_PNG_COMPRESSION,
            outputFormat === 'jpg' ? quality : 9 - Math.floor(quality / 10)
        ];
        await opencv4nodejs_1.default.imwriteAsync(outputPath, image, compressionParams);
        return outputPath;
    }
    generateOutputPath(inputPath, outputFormat) {
        const dirName = path_1.default.dirname(inputPath);
        const baseName = path_1.default.basename(inputPath, path_1.default.extname(inputPath));
        return path_1.default.join(dirName, `${baseName}-converted.${outputFormat}`);
    }
}
exports.default = new OpenCVService();
