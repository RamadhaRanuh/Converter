import path from 'path';
import sharp from 'sharp';

class ImageService {
  async convertImage(inputPath: string, outputFormat: string, quality = 100) {
    const outputPath = this.generateOutputPath(inputPath, outputFormat);
    
    let processor = sharp(inputPath);
    
    if (outputFormat === 'jpg' || outputFormat === 'jpeg') {
      processor = processor.jpeg({ quality });
    } else if (outputFormat === 'png') {
      // For PNG, lower values mean higher quality (opposite of how you had it)
      const compressionLevel = 10 - Math.floor(quality / 10);
      processor = processor.png({ compressionLevel });
    } else if (outputFormat === 'webp') {
      processor = processor.webp({ quality });
    }
    
    await processor.toFile(outputPath);
    return outputPath;
  }

  generateOutputPath(inputPath: string, outputFormat: string) {
    const dirName = path.dirname(inputPath);
    const baseName = path.basename(inputPath, path.extname(inputPath));
    return path.join(dirName, `${baseName}-converted.${outputFormat}`);
  }
}

export default new ImageService();