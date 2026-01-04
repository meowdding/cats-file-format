package me.owdding.cats.api.compression;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;

public final class CatsCompression {

    private final String name;
    private final byte key;
    private final IoOperator<OutputStream> compressor;
    private final IoOperator<InputStream> decompressor;

    CatsCompression(String name, byte key, IoOperator<OutputStream> compressor, IoOperator<InputStream> decompressor) {
        this.name = name;
        this.key = key;
        this.compressor = compressor;
        this.decompressor = decompressor;
    }

    public byte key() {
        return key;
    }

    public OutputStream compress(OutputStream output) throws IOException {
        return compressor.operate(output);
    }

    public InputStream decompress(InputStream input) throws IOException {
        return decompressor.operate(input);
    }

    @Override
    public String toString() {
        return "CatsCompression(%s)".formatted(name);
    }

    @FunctionalInterface
    interface IoOperator<T> {

        T operate(T input) throws IOException;
    }
}
