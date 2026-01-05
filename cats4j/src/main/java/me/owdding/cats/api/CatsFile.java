package me.owdding.cats.api;

import me.owdding.cats.impl.DataReader;
import me.owdding.cats.impl.EntryImpl;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.Map;

public final class CatsFile {

    private static final int CATS_HEADER = 0x43415453; // "CATS"
    private static final byte VERSION = 1;
    private static final int SIGNATURE_SIZE = Integer.BYTES + Byte.BYTES;

    private final int header;
    private final byte[] data;
    private final Map<String, CatsEntry> entries;

    public CatsFile(Path path) throws IOException {
        this(Files.readAllBytes(path));
    }

    public CatsFile(byte[] data) throws IOException {
        this(data, new DataReader(data));
    }

    private CatsFile(byte[] data, DataReader reader) throws IOException {
        if (reader.readInt() != CATS_HEADER) throw new CatsException("Invalid CATS file header.");
        if (reader.readByte() != VERSION) throw new CatsException("Unsupported CATS file version.");

        var directory = EntryImpl.read(reader);

        this.header = SIGNATURE_SIZE + directory.header();
        this.data = data;
        this.entries = readEntries(data, directory);
        this.entries.put("/", directory);
    }

    public CatsEntry getEntry(String path) {
        return entries.get(path);
    }

    public InputStream getInputStream(CatsEntry.File entry) throws IOException {
        return entry.compression().decompress(
                new ByteArrayInputStream(data, header + entry.offset(), entry.size())
        );
    }

    public InputStream getInputStream(String path) throws IOException {
        if (getEntry(path) instanceof CatsEntry.File file) {
            return getInputStream(file);
        }
        return null;
    }

    private static Map<String, CatsEntry> readEntries(byte[] data, CatsEntry.Directory root) throws IOException {
        int dataLength = SIGNATURE_SIZE + root.header();
        Map<String, CatsEntry> result = new HashMap<>();
        for (var value : root.entries().entrySet()) {
            var key = value.getKey();
            var entry = value.getValue();

            switch (entry) {
                case CatsEntry.Directory directory -> {
                    result.put(key, entry);
                    result.putAll(readEntries(data, directory));
                }
                case CatsEntry.File file -> {
                    if (file.offset() < 0 || file.offset() + file.size() > data.length - dataLength) {
                        throw new CatsException("File entry out of bounds: " + key);
                    }
                    if (file.size() <= 0) {
                        throw new CatsException("File entry is empty: " + key);
                    }

                    result.put(key, entry);
                }
            }
        }
        return result;
    }
}
