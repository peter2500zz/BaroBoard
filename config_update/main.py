import tkinter as tk
from tkinter import filedialog
import json
from uuid import uuid4

from typing import Callable, Any, Union, List, Dict


VERSION = 3


UPLINE = "\033[F"
DOWNLINE = "\033[B"
LINEHEAD = "\033[G"
LINEEND = "\033[D"
CLEAR_LINE = "\033[K"


def option(
        options: List[Any] | Dict[Any, Union[str, List[str]]], 
        title: str = "", 
        *, 
        return_key: bool = False, 
        keep_options_after_select: bool = False,
        general_format: str | None = None,
        option_format: str = "{selection_mark}: {option}",
        options_connect: str = "\n",
        invalid_input_return = None,
        function_for_input: Callable[[str], str] | None = None,
    ) -> Any | None:
    """
    创建一个交互式选项选择器。
    
    参数:
        options: 可以是列表或字典。
               - 如果是列表，将自动为每个选项分配数字索引（显示为1开始，但返回为0开始的索引）。
               - 如果是字典，键为选项内容，值为选择标记(字符串或字符串列表)。
                 当值为列表时，第一项将作为显示内容，但所有项都可触发选择。
        title: 选项标题，默认为空。
        return_key: 如果为True，根据options类型返回不同内容：
                  - 对于列表，返回选项的索引（从0开始）
                  - 对于字典，返回用户输入的选择标记
                  如果为False，返回选中的选项内容。
        keep_options_after_select: 如果为True，选择后保留选项显示。
        general_format: 整体格式字符串，默认为"{title}\n{options}\n"。
        option_format: 每个选项的格式，默认为"{selection_mark}: {option}"。
        options_connect: 选项之间的连接符，默认为换行符。
        invalid_input_return: 当输入无效时返回的值。
        function_for_input: 处理用户输入的函数。
    
    返回:
        根据return_key参数返回不同内容：
        - 如果return_key=True：
          * 对于列表，返回选项的索引（从0开始）
          * 对于字典，返回用户输入的选择标记
        - 如果return_key=False，返回选中的选项内容
        - 如果输入无效，返回invalid_input_return
    """
    
    if isinstance(options, list):
        # 将列表转换为字典，键为选项，值为数字索引(字符串形式)
        # 注意：显示给用户的索引从1开始，但内部使用的索引从0开始
        lined_options = {option: str(i + 1) for i, option in enumerate(options)}
    
    elif isinstance(options, dict):
        # 使用提供的字典，但确保值是列表或字符串
        lined_options = {}
        for option, selection_marks in options.items():
            if isinstance(selection_marks, list):
                if not selection_marks:  # 如果列表为空，跳过
                    continue
                lined_options[option] = selection_marks
            else:
                lined_options[option] = selection_marks
    else:
        raise TypeError("options must be a list or a dict")
    
    # 创建反向映射用于查找
    # reverse_mapping: 从选择标记到选项内容的映射
    # selection_to_index: 从选择标记到索引的映射（仅用于列表类型）
    reverse_mapping = {}
    selection_to_index = {}  
    
    if isinstance(options, list):
        for i, option in enumerate(options):
            # 显示的索引从1开始，但返回的索引从0开始
            mark = str(i + 1)
            reverse_mapping[mark] = option
            selection_to_index[mark] = i  # 存储原始索引（从0开始）
    else:
        for option, marks in lined_options.items():
            if isinstance(marks, list):
                for mark in marks:
                    reverse_mapping[mark] = option
            else:
                reverse_mapping[marks] = option
    
    if general_format is None:
        general_format = ("{title}\n" if title else "") + "{options}\n"
    
    # 准备显示选项
    display_options = []
    for option, marks in lined_options.items():
        # 如果marks是列表，使用第一项作为显示标记
        display_mark = marks[0] if isinstance(marks, list) else marks
        display_options.append(option_format.format(selection_mark=display_mark, option=option))
    
    formatted_options = options_connect.join(display_options)
    lines = len(formatted_options.split("\n"))
    
    if title:
        lines += len(title.split("\n"))
    
    # 显示选项并获取用户输入
    selection = input(
        general_format.format(
            title=title,
            options=formatted_options
        )
    )
    
    # 如果提供了输入处理函数，应用它
    if function_for_input is not None:
        selection = function_for_input(selection)
    
    lines += len(selection.split("\n"))
    
    # 如果不保留选项，清除显示的选项
    if not keep_options_after_select:
        for _ in range(lines):
            print(UPLINE + LINEHEAD + CLEAR_LINE, end="")
    
    # 处理用户选择
    if selection in reverse_mapping:
        selected_option = reverse_mapping[selection]
        
        if return_key:
            if isinstance(options, list):
                # 对于列表，返回索引（从0开始）
                return selection_to_index[selection]
            else:
                # 对于字典，返回选择标记
                return selection
        else:
            # 返回选项内容
            return selected_option
    
    else:
        # 输入无效，返回指定的无效输入返回值
        return invalid_input_return
    

def select_links_json_file():
    root = tk.Tk()
    root.withdraw()  # 隐藏主窗口
    root.attributes('-topmost', True)  # 确保对话框在最前面
    
    try:
        file_path = filedialog.askopenfilename(
            title="选择快捷方式配置文件",
            filetypes=[("配置文件", "*.links.json"), ("所有文件", "*.*")],
            parent=root
        )
            
        return file_path
        
    except Exception:
        return None
    finally:
        root.destroy()  # 确保销毁tkinter窗口


def load_conf(path):
    global title, stop_mark
    
    try:
        conf_file = open(path, "r", encoding="utf-8")
        content = conf_file.readlines()
        conf_file.close()

    except FileNotFoundError:
        title = f"{path} 文件不存在或无法读取"
        return

    try:
        conf: dict = json.loads("".join(content))
    except json.JSONDecodeError:
        title = f"{path} 无法作为json文件读取，请检查是否有语法错误"
        return

    new_conf = {}

    new_conf["version"] = VERSION
    if version := conf.get("version"):
        try:
            version_int = int(version)
        except ValueError:
            print(f"版本号 {version} 不是一个有效的整数，将使用版本号 {VERSION}")
        else:
            if version_int < VERSION:
                print(f"版本号 {version_int} 低于当前版本号，将使用版本号 {VERSION}")
            else:
                new_conf["version"] = version_int
        
    else:
        print(f"未找到版本号，将使用版本号 {VERSION}")

    new_conf["tags"] = []
    if (tags := conf.get("tags")) is not None:
        new_conf["tags"] = tags
    else:
        print("无法获取标签，将初始化标签")
        
    new_conf["program_links"] = []
    if (program_links := conf.get("program_links")) is not None:
        for index, program_link in enumerate(program_links):
            index += 1
            
            new_program_link = {}

            new_program_link["name"] = []
            if (name := program_link.get("name")) is not None:
                if isinstance(name, list):
                    new_program_link["name"] = name
                else:
                    new_program_link["name"] = [str(name)]
            else:
                print(f"无法获取第 {index} 个快捷方式的名称，将使用空名称")
                new_program_link["name"] = []

            new_program_link["icon_path"] = ""
            if (icon_path := program_link.get("icon_path")) is not None:
                new_program_link["icon_path"] = icon_path
            else:
                print(f"无法获取第 {index} 个快捷方式的图标路径，将使用空图标路径")
                new_program_link["icon_path"] = ""

            new_program_link["run_command"] = ""
            if (run_command := program_link.get("run_command")) is not None:
                new_program_link["run_command"] = run_command
            else:
                print(f"无法获取第 {index} 个快捷方式的运行命令，将使用空运行命令")
                new_program_link["run_command"] = ""
                
            new_program_link["tags"] = []
            if (tags := program_link.get("tags")) is not None:
                if isinstance(tags, list):
                    for tag in tags:
                        if tag in new_conf["tags"]:
                            new_program_link["tags"].append(tag)
                        
                else:
                    print(f"无法获取第 {index} 个快捷方式的标签，将使用空标签")
                    new_program_link["tags"] = []
            else:
                print(f"无法获取第 {index} 个快捷方式的标签，将使用空标签")
                new_program_link["tags"] = []
                
            new_program_link["uuid"] = ""
            if uuid := program_link.get("uuid"):
                new_program_link["uuid"] = uuid
            else:
                print(f"无法获取第 {index} 个快捷方式的uuid，将使用随机uuid")
                new_program_link["uuid"] = str(uuid4())
                
            new_conf["program_links"].append(new_program_link)

    else:
        print("无法获取快捷方式，将初始化快捷方式")
        new_conf["program_links"] = []

    if new_conf == conf:
        title = "此配置文件无需更新"
        return

    if option(
        {'Y': ['y', 'yes'], 'n': ['n', 'no']},
        "是否保存更新后的配置文件",
        function_for_input=lambda x: x.lower(),
        general_format="{title}[{options}]: ",
        option_format="{option}",
        options_connect="/",
        invalid_input_return='Y',
        keep_options_after_select=True,
    ) == "Y":
        with open(path, "w", encoding="utf-8") as f:
            json.dump(new_conf, f, ensure_ascii=False, indent=4)

        print("配置文件已更新")
        stop_mark = True

title = "BaroBoard 快捷方式配置文件更新"
stop_mark = False
while not stop_mark:
    match option(
        ["选择配置文件", "手动输入配置文件路径", "退出"], 
        title, 
        return_key=True
    ):
        case 0:
            path = select_links_json_file()

        case 1:
            path = input("输入配置文件路径: ")
            print(f"{UPLINE}{CLEAR_LINE}", end="")

        case 2:
            exit()

    if not path.endswith(".links.json"):
        title = f"{path} 不是一个有效的快捷方式配置文件 (.links.json)"
        continue

    load_conf(path)
