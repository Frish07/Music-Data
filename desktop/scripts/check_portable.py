"""Inspect the PE import table without installing Python packages."""
import pathlib
import struct
import sys

data = pathlib.Path(sys.argv[1]).read_bytes()
u16 = lambda p: struct.unpack_from('<H', data, p)[0]
u32 = lambda p: struct.unpack_from('<I', data, p)[0]
pe = u32(0x3C)
assert data[pe:pe + 4] == b'PE\0\0', 'Not a PE executable'
assert u16(pe + 4) == 0x8664, 'Not Windows x64'
optional = pe + 24
assert u16(optional) == 0x20B, 'Not PE32+'
assert u16(optional + 68) == 2, 'Expected Windows GUI subsystem'
sections = optional + u16(pe + 20)

def offset(rva):
    for index in range(u16(pe + 6)):
        section = sections + index * 40
        start = u32(section + 12)
        if start <= rva < start + max(u32(section + 8), u32(section + 16)):
            return u32(section + 20) + rva - start
    raise ValueError(f'Unmapped RVA {rva:x}')

position = offset(u32(optional + 112 + 8))
imports = []
while any(data[position:position + 20]):
    name_start = offset(u32(position + 12))
    imports.append(data[name_start:data.index(b'\0', name_start)].decode('ascii').lower())
    position += 20

system = {'kernel32.dll', 'user32.dll', 'gdi32.dll', 'comdlg32.dll', 'shell32.dll',
          'ole32.dll', 'oleaut32.dll', 'advapi32.dll', 'bcrypt.dll', 'bcryptprimitives.dll',
          'ntdll.dll', 'userenv.dll', 'ws2_32.dll', 'crypt32.dll', 'secur32.dll',
          'dwmapi.dll', 'comctl32.dll', 'shlwapi.dll', 'ucrtbase.dll', 'msvcrt.dll',
          'winmm.dll', 'version.dll', 'propsys.dll', 'rpcrt4.dll', 'normaliz.dll'}
unexpected = [name for name in imports if name not in system and not name.startswith(('api-ms-win-', 'ext-ms-win-'))]
print('Windows system imports:', ', '.join(imports))
assert not unexpected, f'Non-system DLL dependencies: {unexpected}'
print('Portable check passed: x64 GUI executable, Windows system DLLs only.')
