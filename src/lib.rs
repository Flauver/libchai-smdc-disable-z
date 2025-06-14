use chai::data::{元素, 元素映射, 可编码对象, 数据, 编码信息};
use chai::encoders::编码器;
use chai::objectives::default::默认目标函数;
use chai::objectives::metric::默认指标;
use chai::objectives::目标函数;
use chai::错误;
use serde::Serialize;
use std::fmt::Display;
use std::iter::zip;

pub struct 四码定长编码器 {
    pub 进制: u64,
    pub 编码结果: Vec<编码信息>,
    pub 词列表: Vec<可编码对象>,
    pub 全码空间: Vec<u8>,
    pub 简码空间: Vec<u8>,
    pub 包含元素的词映射: Vec<Vec<usize>>,
    pub 空格: u64,
}

impl 四码定长编码器 {
    pub fn 新建(数据: &数据) -> Result<Self, 错误> {
        let 最大码长 = 4;
        let 词列表 = 数据.词列表.clone();
        let 编码输出 = 词列表.iter().map(编码信息::new).collect();
        let 编码空间大小 = 数据.进制.pow(最大码长 as u32) as usize;
        let 全码空间 = vec![u8::default(); 编码空间大小];
        let 简码空间 = 全码空间.clone();
        let mut 包含元素的词映射 = vec![vec![]; 数据.初始映射.len()];
        for (索引, 词) in 词列表.iter().enumerate() {
            for 元素 in &词.元素序列 {
                包含元素的词映射[*元素].push(索引);
            }
        }
        let 空格 = 数据.键转数字[&'_'];
        Ok(Self {
            进制: 数据.进制,
            编码结果: 编码输出,
            词列表,
            全码空间,
            简码空间,
            包含元素的词映射,
            空格,
        })
    }

    pub fn 重置空间(&mut self) {
        self.全码空间.iter_mut().for_each(|x| {
            *x = 0;
        });
        self.简码空间.iter_mut().for_each(|x| {
            *x = 0;
        });
    }

    #[inline(always)]
    fn 全码规则(词: &可编码对象, 映射: &元素映射, 进制: u64) -> u64 {
        let 元素序列 = &词.元素序列;
        let mut 全码 =
            映射[元素序列[0]] + 映射[元素序列[1]] * 进制;
        if 元素序列.len() >= 3 {
            全码 += 映射[元素序列[2]] * 进制 * 进制;
        }
        if 元素序列.len() >= 4 {
            全码 += 映射[元素序列[3]] * 进制 * 进制 * 进制;
        }
        全码
    }

    fn 输出全码(&mut self, 映射: &元素映射, 移动的元素: &Option<Vec<元素>>) {
        let 编码结果 = &mut self.编码结果;
        let 进制 = self.进制;
        if let Some(移动的元素) = 移动的元素 {
            for 元素 in 移动的元素 {
                for 索引 in &self.包含元素的词映射[*元素] {
                    let 词 = &self.词列表[*索引];
                    let 全码 = 四码定长编码器::全码规则(词, 映射, 进制);
                    编码结果[*索引].全码.原始编码 = 全码;
                }
            }
        } else {
            for (词, 编码信息) in zip(&self.词列表, 编码结果.iter_mut()) {
                let 全码 = 四码定长编码器::全码规则(词, 映射, 进制);
                编码信息.全码.原始编码 = 全码;
            }
        }

        for 编码信息 in 编码结果.iter_mut() {
            let 原始编码 = 编码信息.全码.原始编码;
            let 是否重码 = self.全码空间[原始编码 as usize] > 0;
            // 如果低于四码，则实际编码要补空格
            let 实际编码 = if 原始编码 < 进制 * 进制 {
                原始编码 + self.空格 * 进制 * 进制
            } else if 原始编码 < 进制 * 进制 * 进制 {
                原始编码 + self.空格 * 进制 * 进制 * 进制
            } else {
                原始编码
            };
            编码信息.全码.原始编码候选位置 = self.全码空间[原始编码 as usize];
            编码信息.全码.更新(实际编码, 是否重码);
            self.全码空间[原始编码 as usize] += 1;
        }
    }

    fn 输出简码(&mut self) {
        let 编码结果 = &mut self.编码结果;
        let 进制 = self.进制;
        for (编码信息, 词) in zip(编码结果.iter_mut(), &self.词列表) {
            let 全码原始 = 编码信息.全码.原始编码;
            let 全码实际 = 编码信息.全码.实际编码;
            let 简码信息 = &mut 编码信息.简码;
            if 词.词长 == 1 {
                let 一简原始 = 全码原始 % 进制;
                let 重数 = self.全码空间[一简原始 as usize] + self.简码空间[一简原始 as usize];
                if 重数 == 0 {
                    简码信息.原始编码 = 一简原始;
                    简码信息.原始编码候选位置 = self.简码空间[一简原始 as usize];
                    self.简码空间[一简原始 as usize] += 1;
                    let 一简实际 = 一简原始 + self.空格 * 进制;
                    简码信息.更新(一简实际, false);
                    continue;
                }
                let 二简原始 = 全码原始 % (进制 * 进制);
                let 重数 = self.全码空间[二简原始 as usize] + self.简码空间[二简原始 as usize];
                if 重数 == 0 {
                    简码信息.原始编码 = 二简原始;
                    简码信息.原始编码候选位置 = self.简码空间[二简原始 as usize];
                    self.简码空间[二简原始 as usize] += 1;
                    let 二简实际 = 二简原始 + self.空格 * 进制 * 进制;
                    简码信息.更新(二简实际, false);
                    continue;
                }
            }
            // 多字词以及没有简码的一字词
            let 全码是否重码 = self.简码空间[全码原始 as usize] > 0;
            简码信息.原始编码 = 全码原始;
            简码信息.原始编码候选位置 = self.简码空间[全码原始 as usize];
            self.简码空间[全码原始 as usize] += 1;
            简码信息.更新(全码实际, 全码是否重码);
        }
    }
}

impl 编码器 for 四码定长编码器 {
    fn 编码(
        &mut self,
        映射: &元素映射,
        移动的元素: &Option<Vec<元素>>,
    ) -> &mut Vec<编码信息> {
        self.重置空间();
        self.输出全码(映射, 移动的元素);
        self.输出简码();
        &mut self.编码结果
    }
}

pub struct 测试目标函数 {
    默认目标函数: 默认目标函数,
    进制: u64,
    z: u64,
}

#[derive(Clone, Serialize)]
pub struct 测试指标 {
    默认指标: 默认指标,
    z键使用率: f64,
    一简率: f64,
    二简率: f64,
}

impl Display for 测试指标 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n", self.默认指标)?;
        write!(f, "z键使用率：{}；\n", self.z键使用率)?;
        write!(f, "一简率：{}；", self.一简率)?;
        write!(f, "二简率：{}；", self.二简率)
    }
}

impl 测试目标函数 {
    pub fn 新建(数据: &数据) -> Result<Self, 错误> {
        let 默认目标函数 = 默认目标函数::新建(数据)?;
        let 进制 = 数据.进制;
        let z = 数据.键转数字[&'z'];
        Ok(Self {
            默认目标函数,
            进制,
            z,
        })
    }
}

impl 目标函数 for 测试目标函数 {
    type 目标值 = 测试指标;

    fn 计算(
        &mut self, 编码结果: &mut [编码信息], 映射: &元素映射
    ) -> (Self::目标值, f64) {
        let (默认指标, 损失函数) = self.默认目标函数.计算(编码结果, 映射);
        let mut 总按键数 = 0.0;
        let mut z键按键数 = 0.0;
        let mut 总频 = 0.0;
        let mut 一简频 = 0.0;
        let mut 二简频 = 0.0;
        for 编码结果 in 编码结果.iter() {
            let 简码  = 编码结果.简码.实际编码;
            let mut 部分编码 = 简码;
            while 部分编码 > 0 {
                let 键 = 部分编码 % self.进制;
                部分编码 /= self.进制;
                if 键 == 0 {
                    continue;
                }
                总按键数 += 编码结果.频率 as f64;
                if 键 == self.z {
                    z键按键数 += 编码结果.频率 as f64;
                }
            }
            总频 += 编码结果.频率 as f64;
            if 编码结果.简码.原始编码 < self.进制 {
                一简频 += 编码结果.频率 as f64;
            } else if 编码结果.简码.原始编码 < self.进制 * self.进制 { 
                二简频 += 编码结果.频率 as f64;
            }
        }
        let z键使用率 = z键按键数 / 总按键数;
        let 一简率 = 一简频 / 总频 as f64;
        let 二简率 = 二简频 / 总频 as f64;
        let 损失函数 = 损失函数 + if z键使用率 > 0.0175 { 10.0 } else { 0.0 } + 一简率 * -2.0 + 二简率 * -1.0;
        let 指标 = 测试指标 {
            默认指标,
            z键使用率,
            一简率,
            二简率,
        };
        (指标, 损失函数)
    }
}
