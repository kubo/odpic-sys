from dataclasses import dataclass, field
import glob
import re
import sys
import xml.etree.ElementTree as ET

hints = {
    'dpiAuthMode': {
        'type': 'bitflags',
    },
    'dpiConnCloseMode': {
        'type': 'bitflags',
    },
    'dpiContextCreateParams.defaultDriverName': {
        'type': 'c-string',
    },
    'dpiContextCreateParams.defaultEncoding': {
        'type': 'c-string',
    },
    'dpiContextCreateParams.loadErrorUrl': {
        'type': 'c-string',
    },
    'dpiContextCreateParams.oracleClientLibDir': {
        'type': 'c-string',
        'encoding': 'ansi',
    },
    'dpiContextCreateParams.oracleClientConfigDir': {
        'type': 'c-string',
        'encoding': 'ansi',
    },
    'dpiContextCreateParams.sodaUseJsonDesc': {
        'type': 'boolean',
    },
    'dpiContextCreateParams.useJsonId': {
        'type': 'boolean',
    },
    'dpiCreateMode': {
        'type': 'bitflags'
    },
    'dpiExecMode': {
        'type': 'bitflags'
    },
    'dpiJsonOptions': {
        'type': 'bitflags'
    },
    'dpiOpCode': {
        'type': 'bitflags'
    },
    'dpiPoolCloseMode': {
        'type': 'bitflags'
    },
    'dpiSodaFlags': {
        'type': 'bitflags'
    },
    'dpiStartupMode': {
        'type': 'bitflags'
    },
    'dpiSubscrQOS': {
        'type': 'bitflags'
    },
    'dpiVectorFlags': {
        'type': 'bitflags'
    },
}

@dataclass
class Desc:
    lines: list[str]

    def write_to(self, f, prefix):
        for line in self.lines:
            if len(line) != 0:
                print(prefix, line, file=f)
            else:
                print(file=f)

@dataclass
class MemberInfo:
    name: str
    desc: Desc
    type: str | None = None
    mode: str | None = None
    hints: list[str] | None = None

    def write_to(self, f, prefix):
        print(prefix + '- name:', self.name)
        if self.type is not None:
            print(prefix + '  type:', self.type)
        if self.mode is not None:
            print(prefix + '  mode:', self.mode)
        if self.hints is not None:
            print(prefix + '  hints:')
            for key, value in self.hints.items():
                print(f'{prefix}    {key}: {value}')
        print(prefix + '  desc: |')
        self.desc.write_to(f, prefix + '   ')

@dataclass
class FunctionInfo:
    name: str
    desc: Desc
    round_trips: str
    rettype: str
    params: list[MemberInfo]

    def write_to(self, f):
        print('    - name:', self.name)
        print('      desc: |')
        self.desc.write_to(sys.stdout, '       ')
        print('      round_trips:', self.round_trips)
        print('      return:', self.rettype)
        print('      params:')
        for arg in self.params:
            arg.write_to(sys.stdout, '        ')

@dataclass
class DataTypeInfo:
    name: str
    kind: str
    desc: Desc
    underlying_type: str | None = None
    members: list[MemberInfo] = field(default_factory=list)
    functions: list[FunctionInfo] = field(default_factory=list)
    hints: list[str] | None = None

    def write_to(self, f):
        print('- name:', self.name, file=f)
        print('  kind:', self.kind, file=f)
        if self.underlying_type is not None:
            print('  underlying_type:', self.underlying_type)
        if self.hints is not None:
            print('  hints:')
            for key, value in self.hints.items():
                print(f'    {key}: {value}')
        print('  desc: |', file=f)
        self.desc.write_to(sys.stdout, '   ')
        if len(self.members) > 0:
            print('  members:', file=f)
            for member in self.members:
                member.write_to(sys.stdout, '    ')
        if len(self.functions) > 0:
            print('  functions:', file=f)
            for function in self.functions:
                function.write_to(sys.stdout)
        print(file=f)

class DocInfo:
    REG_PATTERN = re.compile(r"/([^/]+)s/([^/]+).xml$")

    def __init__(self):
        self.read_dpi_header_file()
        self.read_round_trips_file()
        data_types = []
        for fname in sorted(glob.glob('doc/xml/*/dpi*.xml')):
            data_types.append(self.read_xml_file(fname))
        # ad-hoc special handling for dpiData
        functions = None
        for i in range(len(data_types)):
            if data_types[i].name == 'dpiData':
                if data_types[i].kind == 'opaque struct':
                    functions = data_types[i].functions
                    del data_types[i]
                else:
                    data_types[i].functions = functions
                    break
        self.data_types = data_types

    def write_to(self, f):
        for data_type in self.data_types:
            data_type.write_to(f)

    def read_dpi_header_file(self):
        enum_underlying_types = dict()
        enum_underlying_types['dpiJsonOptions'] = 'uint32_t'
        enum_underlying_types['dpiSodaFlags'] = 'uint32_t'
        struct_member_types = dict()
        enum_type_pattern = re.compile(r"^typedef\s+(\w+_t)\s+(\w+);")
        struct_type_pattern = re.compile(r"^(?:typedef )?(?:struct|union)\s+(\w*)\s*{$")
        struct_member_pattern = re.compile(r"^\s+(.+?)\s*(\w+);$")
        struct_type_end_pattern = re.compile(r"^\s*}\s*(\w*);")
        struct_name = None
        members = None
        with open('../odpic-sys/odpi/include/dpi.h') as f:
            for line in f:
                m = enum_type_pattern.search(line)
                if m:
                    enum_underlying_types[m.group(2)] = m.group(1)
                if members is None:
                    m = struct_type_pattern.search(line)
                    if m:
                        struct_name = m.group(1)
                        members = []
                else:
                    m = struct_member_pattern.search(line)
                    if m:
                        members.append((m.group(1), m.group(2)))
                    else:
                        m = struct_type_end_pattern.search(line)
                        if m:
                            struct_member_types[m.group(1) if len(m.group(1)) != 0 else struct_name] = members
                            members = None
        self.enum_underlying_types = enum_underlying_types
        self.struct_member_types = struct_member_types

    def read_round_trips_file(self):
        tree = ET.parse('doc/xml/user_guide/round_trips.xml')
        rows = tree.getroot().findall('.//tbody/row')
        d = dict()
        for row in rows:
            entries = row.findall('entry')
            funcname = self.node_to_str(entries[0]).rstrip('()')
            roundtrip = self.node_to_str(entries[1])
            d[funcname] = roundtrip
        self.round_trip_info = d

    def node_to_str(self, node) -> str:
        return ''.join(node.itertext()).strip()

    trans_table = str.maketrans({
        #'_': '\\_',
        #'*': '\\*',
        '<': '&lt;',
        '>': '&gt;',
    })

    def append_to_list(self, l, node, quote: bool = False):
        if quote:
            l.append('`')
            for text in node.itertext():
                l.append(text)
            l.append('`')
        else:
            for text in node.itertext():
                l.append(text.translate(self.trans_table))

    def paragraph_as_desc(self, node) -> Desc:
        link_list = []
        is_first = True
        lines = []
        for paragraph in node.findall("paragraph"):
            l = []
            if type(paragraph.text) is str:
                l.append(paragraph.text)
            for subnode in paragraph:
                if subnode.tag == 'reference':
                    internal = subnode.get('internal') is not None
                    refuri = subnode.get('refuri')
                    reftitle = subnode.get('reftitle')
                    quote = subnode.get('literal') is not None or subnode.get('internal') is not None
                    l.append('[')
                    if quote and reftitle is not None and '.' in reftitle:
                        l.append(f'`{reftitle}`')
                    else:
                       self.append_to_list(l, subnode, quote=quote)
                    l.append(']')
                    if not internal:
                        l.append(f'({refuri})')
                elif subnode.tag == 'strong':
                    l.append('**')
                    self.append_to_list(l, subnode)
                    l.append('**')
                elif subnode.tag == 'emphasis' or subnode.tag == 'title_reference':
                    l.append('*')
                    self.append_to_list(l, subnode)
                    l.append('*')
                elif subnode.tag == 'literal':
                    self.append_to_list(l, subnode, quote=True)
                else:
                    self.append_to_list(l, subnode)
                if type(subnode.tail) is str:
                    l.append(subnode.tail.translate(self.trans_table))
            if is_first:
                is_first = False
            else:
                lines.append('')
            for line in ''.join(l).splitlines():
                lines.append(line.strip())
        if len(link_list) != 0:
            lines.append('')
        for link in link_list:
            lines.append(link)
        return Desc(lines)

    def read_xml_file(self, filename) -> DataTypeInfo:
        m = self.REG_PATTERN.search(filename)
        file_type = m.group(1)
        type_name = m.group(2)
        tree = ET.parse(filename)
        document_node = tree.getroot()
        section_node = document_node.find('section')
        desc = self.paragraph_as_desc(section_node)
        data_type = DataTypeInfo(type_name, file_type, desc)
        match file_type:
            case 'enum':
                data_type.underlying_type = self.enum_underlying_types[type_name]
                data_type.members = self.node_to_enum_members(section_node)
            case 'function':
                data_type.kind = 'opaque struct'
                data_type.functions = self.node_to_functions(section_node)
            case 'struct' | 'union':
                data_type.members = self.node_to_struct_members(section_node, type_name)
        data_type.hints = hints.get(data_type.name)
        return data_type

    def node_to_enum_members(self, section_node) -> list[MemberInfo]:
        values = []
        for row in section_node.findall('.//tbody/row'):
            entries = row.findall('entry')
            values.append(MemberInfo(self.node_to_str(entries[0]), self.paragraph_as_desc(entries[1])))
        return values

    def node_to_functions(self, section_node):
        c_functions = section_node.findall("./desc[@classes='c function']")
        functions = []
        for desc in c_functions:
            ret_type = []
            parameters = []
            for node in desc.findall('./desc_signature/desc_signature_line/*'):
                if node.tag != 'desc_parameterlist':
                    ret_type.append(''.join(node.itertext()))
                else:
                    for param in node.findall('*'):
                        l = []
                        for n in param.findall('*'):
                            l.append(''.join(n.itertext()))
                        name = l.pop()
                        parameters.append((name, ''.join(l)))
            func_name = ret_type.pop()
            content = desc.find('desc_content')
            func_desc = self.paragraph_as_desc(content)
            round_trips = self.round_trip_info[func_name]
            func_return = ''.join(ret_type)
            rows = content.findall('.//tbody/row')
            if len(rows) != len(parameters):
                raise RuntimeError(f'{len(rows)} != {len(parameters)} in {func_name}')
            args = []
            for (row, param) in zip(rows, parameters):
                entries = row.findall('entry')
                name = self.node_to_str(entries[0])
                if param[0] != name:
                    raise RuntimeError(f'{param[0]} != {name} in {func_name}')
                param_desc = self.paragraph_as_desc(entries[2])
                args.append(MemberInfo(name, param_desc, param[1], self.node_to_str(entries[1])))
            functions.append(FunctionInfo(func_name, func_desc, round_trips, func_return, args))
        return functions

    def node_to_struct_members(self, section_node, type_name):
        pos = 0
        members = []
        for node in section_node.findall("./desc[@classes='c member']"):
            l = []
            for n in node.findall('.//desc_signature_line/*'):
                if n.tag not in ('desc_addname', 'desc_name'):
                    l.append(''.join(n.itertext()))
            member_type = ''.join(l).strip()
            member_name = node.find('.//desc_name/desc_sig_name').text
            member_desc = self.paragraph_as_desc(node.find('./desc_content'))
            # Remove this when ODPI-C 5.4.2 is released.
            match (type_name, member_name, member_type):
                case ('dpiDataTypeInfo', 'fsPrecision', 'int16_t'):
                    member_type = 'uint8_t'
                case ('dpiDataTypeInfo', 'annotations', 'uint32_t'):
                    member_type = 'dpiAnnotation *'
                case ('dpiErrorInfo', 'offset', 'uint16_t'):
                    member_type = 'uint32_t'
                case ('dpiDataBuffer', 'asJson', 'dpiJsonNode'):
                    member_type = 'dpiJson *'
                case ('dpiDataBuffer', 'asJsonObject', 'dpiJsonNode'):
                    member_type = 'dpiJsonObject'
                case ('dpiDataBuffer', 'asJsonArray', 'dpiJsonNode'):
                    member_type = 'dpiJsonArray'
                case ('dpiDataBuffer', 'asObject', 'int'):
                    member_type = 'dpiObject *'
                case ('dpiDataBuffer', 'asStmt', 'int'):
                    member_type = 'dpiStmt *'
                case ('dpiDataBuffer', 'asRowid', 'int'):
                    member_type = 'dpiRowid *'

            member = MemberInfo(member_name, member_desc, member_type)
            member.hints = hints.get(f'{type_name}.{member_name}')
            members.append(member)

        # Ensure that struct member types in documents are same with ones in dpi.h.
        if type_name != 'dpiStringList':
            members_in_dpi_h = self.struct_member_types[type_name]
            if len(members) != len(members_in_dpi_h):
                raise RuntimeError(f'{len(members)} != {len(members_in_dpi_h)} for struct {type_name}')
            for (m1, m2) in zip(members, members_in_dpi_h):
                if m1.name != m2[1]:
                    raise RuntimeError(f"mismatched member name of struct {type_name}: '{m1.name}' != '{m2[1]}'")
                if m1.type.replace(' ', '') != m2[0].replace(' ', ''):
                    raise RuntimeError(f"mismatched member type of struct {type_name}: '{m1.type}' != {m2[0]}'")
        return members

def main():
    docinfo = DocInfo()
    docinfo.write_to(sys.stdout)

if __name__ == '__main__':
    sys.exit(main())
