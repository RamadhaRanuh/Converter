import PSD from 'psd';
import path from 'path';
import sharp from 'sharp';
import fs from 'fs';
import { execSync } from 'child_process';

class PSDService {
    // Convert PSD to other formats
    async convertPSD(inputPath: string, outputFormat: any, quality = 100) {
        // If input is already not a PSD file, use createPSD instead
        if (path.extname(inputPath).toLowerCase() !== '.psd') {
            return await this.createPSD(inputPath);
        }
        
        const psd = await PSD.open(inputPath);
        const outputPath = this.generateOutputPath(inputPath, outputFormat);

        // Convert to PNG first
        const pngBuffer: any | undefined = psd.image?.toPng();

        // Use sharp for final conversion
        await sharp(pngBuffer)
            .toFormat(outputFormat, { quality })
            .toFile(outputPath);

        return outputPath;
    }
    
    // Create PSD from other image formats
    async createPSD(inputPath: string) {
        const outputPath = this.generateOutputPath(inputPath, 'psd');
        
        try {
            // Check if ImageMagick is installed
            execSync('magick --version', { stdio: 'ignore' });
            
            console.log(`Converting ${inputPath} to PSD using ImageMagick`);
            execSync(`magick "${inputPath}" "${outputPath}"`, { 
                stdio: 'inherit',
                windowsHide: true 
            });
            
            return outputPath;
        } catch (error) {
            console.error('ImageMagick conversion failed:', error);
            
            // If ImageMagick isn't available or failed, throw a helpful error
            throw new Error(
                `Converting to PSD requires ImageMagick to be installed.\n` +
                `Please install ImageMagick (https://imagemagick.org/script/download.php) ` +
                `and ensure it's in your PATH.`
            );
        }
    }

    // Generate output path
    async extractLayers(inputPath: any, outputFormat: any) {
        const psd = await PSD.open(inputPath);
        const outputDir = path.join(
            path.dirname(inputPath),
            path.basename(inputPath, path.extname(inputPath)) + '-layers'
        );

        if(!fs.existsSync(outputDir)) {
            fs.mkdirSync(outputDir, { recursive: true});
        }

        const layers: any = [];
        const tree = psd.tree();

        const processNode = async (node: any, index: any) => {
            if(node.isGroup()) {
                for (let i = 0; i < node.children().length; i++) {
                    await processNode(node.children()[i], `${index}-${i}`);
                }
            } else {
                if (node.visible) {
                    const layerPath = path.join(outputDir, `layer-${index}.${outputFormat}`);
                    await sharp(node.toPng())
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

    generateOutputPath(inputPath: any, outputFormat: any) {
        const dirName = path.dirname(inputPath);
        const baseName = path.basename(inputPath, path.extname(inputPath));
        return path.join(dirName, `${baseName}-converted.${outputFormat}`);
    }
}

export default new PSDService();