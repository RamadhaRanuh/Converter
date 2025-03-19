"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const svgo_1 = __importDefault(require("svgo"));
const promises_1 = __importDefault(require("fs/promises"));
const path_1 = __importDefault(require("path"));
class SVGService {
    async optimizeSVG(inputPath) {
        const svgString = await promises_1.default.readFile(inputPath, 'utf8');
        const outputPath = this.generateOutputPath(inputPath);
        const result = svgo_1.default.optimize(svgString, {
            plugins: [
                'removeDoctype',
                'removeXMLProcInst',
                'removeComments',
                'removeMetadata',
                'removeEditorsNSData',
                'cleanupAttrs',
                'minifyStyles',
                'convertStyleToAttrs',
                'cleanupIDs',
                'removeUselessDefs',
                'cleanupNumericValues',
                'convertColors',
                'removeUnknownsAndDefaults',
                'removeNonInheritableGroupAttrs',
                'removeUselessStrokeAndFill',
                'removeViewBox',
                'cleanupEnableBackground',
                'removeHiddenElems',
                'removeEmptyText',
                'convertShapeToPath',
                'convertEllipseToCircle',
                'moveElemsAttrsToGroup',
                'moveGroupAttrsToElems',
                'collapseGroups',
                'convertPathData',
                'convertTransform',
                'removeEmptyAttrs',
                'removeEmptyContainers',
                'mergePaths',
                'removeUnusedNS',
                'sortDefsChildren',
                'removeTitle',
                'removeDesc'
            ]
        });
        if ('data' in result) {
            await promises_1.default.writeFile(outputPath, result.data);
            return outputPath;
        }
        else {
            throw new Error(`SVG optimization failed: ${result.error}`);
        }
    }
    generateOutputPath(inputPath) {
        const dirName = path_1.default.dirname(inputPath);
        const baseName = path_1.default.basename(inputPath, path_1.default.extname(inputPath));
        return path_1.default.join(dirName, `${baseName}-optimized.svg`);
    }
}
exports.default = new SVGService();
