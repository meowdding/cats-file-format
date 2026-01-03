package me.owdding.cats.api.compression;

import java.util.zip.GZIPInputStream;
import java.util.zip.GZIPOutputStream;

public final class CatsCompressions {

    public static final byte KEY_NONE = (byte) 0xFF;
    public static final CatsCompression NONE = new CatsCompression(
            KEY_NONE, output -> output, input -> input
    );

    public static final byte KEY_GZIP = (byte) 0x01;
    public static final CatsCompression GZIP = new CatsCompression(
            KEY_GZIP, GZIPOutputStream::new, GZIPInputStream::new
    );

}
