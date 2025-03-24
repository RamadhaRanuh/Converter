import path from 'path';
import OpenCVService from './imageService';
import PSDService from './psdService';
import SVGService from './svgService';
import AIService from './aiService'; 
import util from 'util';
import os from 'os';
import { exec } from 'child_process';

const execPromise = util.promisify(exec);

interface ConversionOptions {
    quality?: number;
}

class ConverterService {
  async convertImage(filePath: string, targetFormat: string, options: ConversionOptions = {}) {
    const ext = path.extname(filePath).toLowerCase();
    try {
      // First check if it's a PSD file that needs conversion before processing
      if (ext === '.psd' && targetFormat !== 'psd') {
        // Convert PSD to PNG first for further processing
        console.log(`Converting PSD to PNG first: ${filePath}`);
        const tempPngPath = await this.convertPsdToPng(filePath);
        console.log(`Converted to temporary PNG: ${tempPngPath}`);
        filePath = tempPngPath; // Use the PNG for further processing
      }

      if (!require('fs').existsSync(filePath)) {
        throw new Error(`Input file not found: ${filePath}`);
      }
      
  console.log(`Processing: ${filePath} -> ${targetFormat}`);
    try {
      // Route to appropriate service based on file extension
      switch (targetFormat) {
        case 'psd':
          console.log('psd go here');
          return await PSDService.convertPSD(filePath, targetFormat, options.quality);
        
        case 'svg':
          if (ext === '.svg') {
            // Optimize SVG using SVGO
            console.log('svg go here')
            return await SVGService.optimizeSVG(filePath); 
          } else {
            return await SVGService.traceSVG(filePath);
          }
        
        case 'ai':
          console.log('ai goes here');
          return await AIService.convertAI(filePath, targetFormat);
        
        case 'jpg':
        case 'jpeg':
        case 'png':
        case 'bmp':
        case 'webp':
        case 'tiff':
          console.log('pixel go here');
          return await OpenCVService.convertImage(filePath, targetFormat, options.quality);
        
        default:
          throw new Error(`Unsupported file format: ${ext}`);
      }
    } catch (error) {
      console.error('Conversion error:', error);
      throw error;
    }
  } catch (error) {
    console.error('Conversion error:', error);
    throw error;
  }
}

async convertPsdToPng(psdPath: string): Promise<string> {
  try {
      // Create output path in the same format as generateOutputPath
      const dirName = path.dirname(psdPath);
      const baseName = path.basename(psdPath, path.extname(psdPath));
      const outputPngPath = path.join(dirName, `${baseName}-converted.png`);
      
      // Use ImageMagick to convert PSD to PNG
      console.log(`Converting PSD: "${psdPath}" to "${outputPngPath}"`);
      await execPromise(`magick convert "${psdPath}[0]" "${outputPngPath}"`);
      
      return outputPngPath;
  } catch (error: any) {
      console.error('Failed to convert PSD to PNG:', error);
      throw new Error(`PSD conversion failed: ${error.message}`);
  }
}

  async extractPSDLayers(filePath: string, targetFormat: string) {
    try {
      return await PSDService.extractLayers(filePath, targetFormat);
    } catch (error) {
      console.error('PSD layer extraction error:', error);
      throw error;
    }
  }
  
}

export default new ConverterService();