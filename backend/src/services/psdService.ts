import PSD from 'psd';
import path from 'path';
import sharp from 'sharp';
import fs from 'fs';

class PSDService {
    // Convert PSD to PNG
    async convertPSD(inputPath: any, outputFormat: any, quality = 100) {
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