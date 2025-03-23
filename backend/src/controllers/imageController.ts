import fs from 'fs';
import path from 'path';
import ConverterService from '../services/converterService';

// Create a named export object
export const ImageController = {
  uploadImage: async (req: any, res: any) => {
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
    } catch (error: string | any) {
      console.error('Upload error:', error);
      return res.status(500).json({ error: error.message });
    }
  },

  convertImage: async (req: any, res: any) => {
    try {
      const { filePath, targetFormat, quality } = req.body;
      
      if (!filePath || !targetFormat) {
        return res.status(400).json({ error: 'File path and target format are required' });
      }
      
      const absFilePath = path.join(process.cwd(), filePath);
      
      if (!fs.existsSync(absFilePath)) {
        return res.status(404).json({ error: 'File not found' });
      }
      
      const convertedPath = await ConverterService.convertImage(absFilePath, targetFormat, {
        quality: parseInt(quality) || 90
      });
      
      return res.status(200).json({
        message: 'File converted successfully',
        originalPath: filePath,
        convertedPath: path.relative(process.cwd(), convertedPath as string),
        targetFormat
      });
    } catch (error: string | any) {
      console.error('Conversion error:', error);
      return res.status(500).json({ error: error.message });
    }
  },

  extractPSDLayers: async (req: any, res: any) => {
    try {
      const { filePath, targetFormat } = req.body;
      
      if (!filePath) {
        return res.status(400).json({ error: 'File path is required' });
      }
      
      const absFilePath = path.join(process.cwd(), filePath);
      
      if (!fs.existsSync(absFilePath)) {
        return res.status(404).json({ error: 'File not found' });
      }

      interface PSDLayer {
          name: string;
          path: string;
      }
      
      const result = await ConverterService.extractPSDLayers(absFilePath, targetFormat || 'png');
      
      return res.status(200).json({
        message: 'PSD layers extracted successfully',
        originalPath: filePath,
        layersDir: path.relative(process.cwd(), result.layersDir),
        layers: result.layers.map((layer: PSDLayer) => ({
          ...layer,
          path: path.relative(process.cwd(), layer.path)
        }))
      });
    } catch (error: string | any) {
      console.error('Layer extraction error:', error);
      return res.status(500).json({ error: error.message });
    }
  }
};

// You can also export the functions individually if needed
export const { uploadImage, convertImage, extractPSDLayers } = ImageController;