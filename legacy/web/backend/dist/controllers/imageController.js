"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.extractPSDLayers = exports.convertImage = exports.uploadImage = exports.ImageController = void 0;
const fs_1 = __importDefault(require("fs"));
const path_1 = __importDefault(require("path"));
const converterService_1 = __importDefault(require("../services/converterService"));
// Create a named export object
exports.ImageController = {
    uploadImage: async (req, res) => {
        try {
            if (!req.file) {
                return res.status(400).json({ error: 'No file uploaded' });
            }
            return res.status(200).json({
                message: 'File uploaded successfully',
                file: {
                    filename: req.file.filename,
                    originalname: req.file.originalname,
                    mimetype: req.file.mimetype,
                    size: req.file.size,
                    path: req.file.path
                }
            });
        }
        catch (error) {
            console.error('Upload error:', error);
            return res.status(500).json({ error: error.message });
        }
    },
    convertImage: async (req, res) => {
        try {
            const { filePath, targetFormat, quality } = req.body;
            if (!filePath || !targetFormat) {
                return res.status(400).json({ error: 'File path and target format are required' });
            }
            const absFilePath = path_1.default.join(process.cwd(), filePath);
            if (!fs_1.default.existsSync(absFilePath)) {
                return res.status(404).json({ error: 'File not found' });
            }
            const convertedPath = await converterService_1.default.convertImage(absFilePath, targetFormat, {
                quality: parseInt(quality) || 90
            });
            return res.status(200).json({
                message: 'File converted successfully',
                originalPath: filePath,
                convertedPath: path_1.default.relative(process.cwd(), convertedPath),
                targetFormat
            });
        }
        catch (error) {
            console.error('Conversion error:', error);
            return res.status(500).json({ error: error.message });
        }
    },
    extractPSDLayers: async (req, res) => {
        try {
            const { filePath, targetFormat } = req.body;
            if (!filePath) {
                return res.status(400).json({ error: 'File path is required' });
            }
            const absFilePath = path_1.default.join(process.cwd(), filePath);
            if (!fs_1.default.existsSync(absFilePath)) {
                return res.status(404).json({ error: 'File not found' });
            }
            const result = await converterService_1.default.extractPSDLayers(absFilePath, targetFormat || 'png');
            return res.status(200).json({
                message: 'PSD layers extracted successfully',
                originalPath: filePath,
                layersDir: path_1.default.relative(process.cwd(), result.layersDir),
                layers: result.layers.map((layer) => ({
                    ...layer,
                    path: path_1.default.relative(process.cwd(), layer.path)
                }))
            });
        }
        catch (error) {
            console.error('Layer extraction error:', error);
            return res.status(500).json({ error: error.message });
        }
    }
};
// You can also export the functions individually if needed
exports.uploadImage = exports.ImageController.uploadImage, exports.convertImage = exports.ImageController.convertImage, exports.extractPSDLayers = exports.ImageController.extractPSDLayers;
