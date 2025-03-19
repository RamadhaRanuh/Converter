"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const child_process_1 = require("child_process");
const path_1 = __importDefault(require("path"));
const util_1 = require("util");
class AIService {
    constructor() {
        this.execPromise = (0, util_1.promisify)(child_process_1.exec);
    }
    async convertAI(inputPath, outputFormat) {
        const outputPath = this.generateOutputPath(inputPath, outputFormat);
        // Using ImageMagick
        try {
            await this.execPromise(`convert ${inputPath} ${outputPath}`);
            return outputPath;
        }
        catch (error) {
            try {
                await this.execPromise(`inkscape ${inputPath} --export-filename=${outputPath}`);
                return outputPath;
            }
            catch (inkscapeError) {
                throw new Error('Failed to convert AI file');
            }
        }
    }
    generateOutputPath(inputPath, outputFormat) {
        const dirName = path_1.default.dirname(inputPath);
        const baseName = path_1.default.basename(inputPath, path_1.default.extname(inputPath));
        return path_1.default.join(dirName, `${baseName}.${outputFormat}`);
    }
}
exports.default = new AIService();
